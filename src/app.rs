use std::{
    error, io,
    iter::zip,
    mem,
    net::{AddrParseError, Ipv4Addr, TcpStream},
    num::ParseIntError,
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
};

use derive_more::{Display, From};
use egui::{
    Align, Button, CentralPanel, Context, Frame, Layout, PopupCloseBehavior, RichText, SidePanel,
    TopBottomPanel,
    containers::menu::{MenuBar, MenuConfig},
};
use egui_notify::Toasts;
use log::{debug, error};

use super::{
    channel::{Channel, ProtocolError},
    listener::{PendingConnection, listener_thread},
    protocol::Message,
};

const OUR_NAME: &str = "You";

pub struct GrapevineApp {
    // core app
    channels: Arc<Mutex<Vec<Arc<Channel>>>>,
    pending_connections: Arc<Mutex<Vec<PendingConnection>>>,
    selected_channel: Option<Arc<Channel>>,
    // UI related
    channel_name_input: String,
    channel_ip_input: String,
    channel_port_input: String,
    channel_message_input: String,
    // utils
    toasts: Toasts,
}

impl GrapevineApp {
    pub fn new(address: Ipv4Addr, port: u16) -> Self {
        let pending_connections = Arc::new(Mutex::new(Vec::new()));

        let res = Self {
            channels: Arc::new(Mutex::new(Vec::new())),
            pending_connections: pending_connections.clone(),
            channel_name_input: String::new(),
            channel_ip_input: String::new(),
            channel_port_input: String::new(),
            channel_message_input: String::new(),
            selected_channel: None,
            toasts: Toasts::new(),
        };

        thread::spawn(move || {
            debug!("Starting listener thread for {}:{}", address, port);
            listener_thread((address, port), pending_connections)
        });

        return res;
    }

    fn new_channel(
        &mut self,
        ip: Ipv4Addr,
        port: u16,
        name: Option<String>,
    ) -> Result<(), io::Error> {
        let stream = TcpStream::connect((ip, port))?;

        let channels = self.channels.clone();
        thread::spawn(move || -> Result<(), ChannelFormError> {
            let channel =
                Arc::new(Channel::new(stream, name).ok_or(ProtocolError::VerificationError)?);

            channels.lock().unwrap().push(channel.clone());
            debug!("Starting listening thread for channel {}", channel.name());
            channel.listen()?;
            Ok(())
        });
        Ok(())
    }
}

impl Default for GrapevineApp {
    fn default() -> Self {
        Self::new(Ipv4Addr::new(0, 0, 0, 0), 5000)
    }
}

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

impl eframe::App for GrapevineApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        SidePanel::left("Channels").show(ctx, |ui| {
            // well all this menu bar business is horribly convoluted
            MenuBar::new()
                .config(
                    MenuConfig::default().close_behavior(PopupCloseBehavior::CloseOnClickOutside),
                )
                .ui(ui, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        for channel in self.channels.lock().unwrap().iter() {
                            let selected =
                                self.selected_channel.as_ref().is_some_and(|c| c == channel);
                            let button =
                                Button::new(RichText::new(channel.name())).selected(selected);
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
                                    let name = Some(self.channel_name_input.clone())
                                        .filter(|s| !s.is_empty());

                                    self.new_channel(ip, port, name)?;

                                    ui.close();
                                }
                                Ok(())
                            })
                            .inner
                            .transpose()
                        {
                            let err_msg = format!("Connection error: {}", e);
                            error!("{}", err_msg);
                            self.toasts.error(&err_msg);
                        }

                        for pending in self.pending_connections.lock().unwrap().iter() {
                            Frame::group(ui.style())
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(pending.name());

                                        if ui.small_button("✔").clicked() {
                                            return Some(true);
                                        }

                                        if ui.small_button("✘").clicked() {
                                            return Some(false);
                                        }
                                        None
                                    })
                                    .inner
                                })
                                .inner;
                        }
                    })
                });
        });

        CentralPanel::default().show(ctx, |ui| {
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
                            .on_hover_text(
                                message.timestamp().format("%Y-%m-%d %H:%M:%S").to_string(),
                            );
                    });
                }

                TopBottomPanel::bottom("message_panel").show(ctx, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        let resp = ui.text_edit_singleline(&mut self.channel_message_input);
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if !self.channel_message_input.is_empty() {
                                let message =
                                    Message::new(mem::take(&mut self.channel_message_input));
                                if let Err(e) = channel.send_message(message) {
                                    error!("Message sending error: {}", e);
                                    self.toasts.error(&format!("Message sending error: {}", e));
                                }
                            }
                            resp.request_focus();
                        }
                    })
                });
            }
        });

        self.toasts.show(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}
}
