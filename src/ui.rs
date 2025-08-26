use std::{
    error, io, mem,
    net::{AddrParseError, Ipv4Addr},
    num::ParseIntError,
    str::FromStr,
    sync::{Arc, Mutex},
};

use derive_more::{Display, From};
use egui::{
    Align, Button, CentralPanel, Context, Frame, Layout, PopupCloseBehavior, RichText, SidePanel,
    TopBottomPanel, Ui,
    containers::menu::{MenuBar, MenuConfig},
};

use super::{
    app::{Channel, GrapevineApp, ProtocolError},
    handler::UiEventHandler,
    protocol::Message,
};

const OUR_NAME: &str = "You";

#[derive(Debug, From, Display)]
enum ChannelFormError {
    InvalidPort(ParseIntError),
    InvalidIp(AddrParseError),
    IoError(io::Error),
    ProtocolError(ProtocolError),
}

impl error::Error for ChannelFormError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidIp(e) => Some(e),
            Self::InvalidPort(e) => Some(e),
            Self::IoError(e) => Some(e),
            Self::ProtocolError(e) => Some(e),
        }
    }
}

pub struct GrapevineUI {
    // encapsulations
    app: GrapevineApp,
    event_handler: Arc<Mutex<UiEventHandler>>,
    // input
    channel_name_input: String,
    channel_ip_input: String,
    channel_port_input: String,
    channel_message_input: String,
    // Vis
    selected_channel: Option<Arc<Channel>>,
}

impl GrapevineUI {
    pub fn new(mut app: GrapevineApp) -> Self {
        let event_handler = Arc::new(Mutex::new(UiEventHandler::default()));

        app.add_event_recipient(event_handler.clone());

        Self {
            app,
            event_handler,
            channel_name_input: String::new(),
            channel_ip_input: String::new(),
            channel_port_input: String::new(),
            channel_message_input: String::new(),
            selected_channel: None,
        }
    }
}

impl GrapevineUI {
    fn channels_panel(&mut self, ui: &mut Ui) {
        for channel in self.app.channels().lock().unwrap().iter() {
            let selected = self.selected_channel.as_ref().is_some_and(|c| c == channel);
            let button = Button::new(RichText::new(channel.name())).selected(selected);
            if ui.add(button).clicked() {
                self.selected_channel = Some(channel.clone());
            }
        }

        if let Err(e) = ui
            .menu_button("New Channel", |ui| -> Result<(), ChannelFormError> {
                ui.label("Channel Name");
                ui.text_edit_singleline(&mut self.channel_name_input);

                ui.label("IP");
                ui.text_edit_singleline(&mut self.channel_ip_input);

                ui.label("Port");
                ui.text_edit_singleline(&mut self.channel_port_input);

                if ui.button("Create").clicked() {
                    let port: u16 = self.channel_port_input.parse()?;
                    let ip = Ipv4Addr::from_str(&self.channel_ip_input)?;
                    let name = Some(self.channel_name_input.clone()).filter(|s| !s.is_empty());

                    self.app.new_channel(ip, port, name)?;

                    ui.close();
                }
                Ok(())
            })
            .inner
            .transpose()
        {
            self.event_handler
                .lock()
                .unwrap()
                .error(&format!("Connection error: {}", e));
        }

        // first we clear the pending connections
        for pending in self.app.inspect_pending() {
            Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(pending.name());

                    if ui.small_button("✔").clicked() {
                        match self.app.add_channel(pending, None) {
                            Ok(_) => self.event_handler.lock().unwrap().info("Channel added"),
                            Err(e) => self
                                .event_handler
                                .lock()
                                .unwrap()
                                .error(&format!("Failed to add channel: {}", e)),
                        };
                    } else if ui.small_button("✘").clicked() {
                        pending.reject();
                        self.event_handler
                            .lock()
                            .unwrap()
                            .info("Connection rejected");
                    } else {
                        // keep the pending connection if nothing was done
                        self.app.add_pending(pending);
                    }
                })
            });
        }
    }

    fn central_panel(&mut self, ctx: &Context, ui: &mut Ui) {
        if let Some(channel) = &self.selected_channel {
            for message in channel.messages().lock().unwrap().iter() {
                let (author, layout) = if message.is_theirs() {
                    (channel.name(), Layout::left_to_right(Align::TOP))
                } else {
                    (OUR_NAME, Layout::right_to_left(Align::TOP))
                };
                let text = format!("{}: {}", author, message.content());

                ui.with_layout(layout, |ui| {
                    Frame::group(ui.style())
                        .show(ui, |ui| {
                            ui.label(text);
                        })
                        .response
                        .on_hover_text(message.timestamp().format("%Y-%m-%d %H:%M:%S").to_string());
                });
            }

            TopBottomPanel::bottom("message_panel").show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    let resp = ui.text_edit_singleline(&mut self.channel_message_input);
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.channel_message_input.is_empty() {
                            let message = Message::new(mem::take(&mut self.channel_message_input));
                            if let Err(e) = channel.send_message(message) {
                                self.event_handler
                                    .lock()
                                    .unwrap()
                                    .error(&format!("Message sending error: {}", e));
                            }
                        }
                        resp.request_focus();
                    }
                })
            });
        }
    }
}

impl eframe::App for GrapevineUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        SidePanel::left("Channels").show(ctx, |ui| {
            // well all this menu bar business is horribly convoluted
            MenuBar::new()
                .config(
                    MenuConfig::default().close_behavior(PopupCloseBehavior::CloseOnClickOutside),
                )
                .ui(ui, |ui| {
                    ui.vertical_centered_justified(|ui| self.channels_panel(ui))
                });
        });

        CentralPanel::default().show(ctx, |ui| self.central_panel(ctx, ui));

        self.event_handler.lock().unwrap().ui(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}
}
