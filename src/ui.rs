use std::{
    mem,
    sync::{Arc, Mutex},
};

use egui::{
    Align, Button, CentralPanel, Context, Frame, Layout, PopupCloseBehavior, RichText, ScrollArea,
    SidePanel, TopBottomPanel, Ui,
    containers::menu::{MenuBar, MenuConfig},
};

use super::{
    app::{Channel, GrapevineApp, PendingConnection},
    handler::UiEventHandler,
    modals::{
        ChannelAcceptAesForm, ChannelAcceptRsaForm, ChannelArgs, ChannelForm, ModalForm,
        SettingsForm,
    },
    protocol::Message,
    settings::Settings,
};

pub struct GrapevineUI {
    // encapsulations
    app: GrapevineApp,
    event_handler: Arc<Mutex<UiEventHandler>>,
    channel_message_input: String,
    // Vis
    selected_channel: Option<Arc<Channel>>,
    settings_modal: Option<ModalForm<SettingsForm>>,
    channel_modal: Option<ModalForm<ChannelForm>>,
    channel_rsa_modal: Option<ModalForm<ChannelAcceptRsaForm>>,
    channel_aes_modal: Option<ModalForm<ChannelAcceptAesForm>>,
    settings: Settings,
}

impl GrapevineUI {
    pub fn new() -> Self {
        let event_handler = Arc::new(Mutex::new(UiEventHandler::default()));
        let settings = Settings::default();

        let mut app = GrapevineApp::new();

        app.add_event_recipient(event_handler.clone());
        if let Some(addr) = settings.listening() {
            app.start_listening(addr.clone());
        }

        Self {
            app,
            event_handler,
            selected_channel: None,
            channel_message_input: String::new(),
            settings_modal: None,
            channel_modal: None,
            channel_rsa_modal: None,
            channel_aes_modal: None,
            settings: settings,
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

        if ui.button("Create channel").clicked() {
            self.channel_modal = Some(ModalForm::new(ChannelForm::default(), "New Channel"));
        }

        // first we clear the pending connections
        for pending in self.app.inspect_pending() {
            Frame::group(ui.style()).show(ui, |ui| {
                let width = ui.available_width();
                ui.horizontal_centered(|ui| {
                    ui.set_min_width(width);
                    ui.label(pending.name());

                    let label = match pending {
                        PendingConnection::Aes(_) => "?",
                        PendingConnection::Rsa(_) => "✔",
                    };

                    if ui.small_button(label).clicked() {
                        match pending {
                            PendingConnection::Aes(aes) => {
                                self.channel_aes_modal = Some(ModalForm::new(
                                    ChannelAcceptAesForm::new(aes),
                                    "Aes Accept",
                                ))
                            }
                            PendingConnection::Rsa(rsa) => {
                                self.channel_rsa_modal = Some(ModalForm::new(
                                    ChannelAcceptRsaForm::new(rsa),
                                    "Rsa Accept",
                                ));
                            }
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
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for message in channel.messages().lock().unwrap().iter() {
                        let (author, layout) = if message.is_ours() {
                            (self.settings.username(), Layout::right_to_left(Align::TOP))
                        } else {
                            (channel.name(), Layout::left_to_right(Align::TOP))
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
                });

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

    fn top_panel(&mut self, ui: &mut Ui) {
        if ui.button("Settings").clicked() {
            self.settings_modal = Some(ModalForm::new(
                SettingsForm::new(&self.settings),
                "Settings",
            ));
        }
    }
}

impl eframe::App for GrapevineUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("Options Panel")
            .frame(
                Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(0),
            )
            .resizable(false)
            .show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Max), |ui| self.top_panel(ui))
            });

        SidePanel::left("Channels")
            .resizable(false)
            .show(ctx, |ui| {
                // well all this menu bar business is horribly convoluted
                MenuBar::new()
                    .config(
                        MenuConfig::default()
                            .close_behavior(PopupCloseBehavior::CloseOnClickOutside),
                    )
                    .ui(ui, |ui| {
                        ui.vertical_centered_justified(|ui| self.channels_panel(ui))
                    });
            });

        CentralPanel::default().show(ctx, |ui| self.central_panel(ctx, ui));

        if let Some(settings) = self
            .settings_modal
            .as_mut()
            .and_then(|modal| modal.show(ctx))
        {
            self.settings = settings;
            if let Some(addr) = self.settings.listening() {
                self.app.start_listening(addr.clone());
            } else {
                self.app.stop_listening();
            }

            self.settings_modal = None;
        }

        if let Some(ret) = self
            .channel_modal
            .as_mut()
            .and_then(|modal| modal.show(ctx))
        {
            if let Some(args) = ret {
                if let Err(e) = match args {
                    ChannelArgs::Rsa(rsa) => self.app.new_rsa_channel(rsa.0, rsa.1),
                    ChannelArgs::Aes(aes) => self.app.new_aes_channel(aes.0, aes.2, aes.3, aes.1),
                } {
                    self.event_handler
                        .lock()
                        .unwrap()
                        .error(format!("Error adding channel: {}", e));
                }
            }
            self.channel_modal = None;
        }

        if let Some(res) = self
            .channel_rsa_modal
            .as_mut()
            .and_then(|modal| modal.show(ctx))
        {
            let pending = self.channel_rsa_modal.take().unwrap().inner().pending();
            if let Some(name) = res {
                if let Err(e) = self.app.add_rsa_channel(pending, name) {
                    self.event_handler
                        .lock()
                        .unwrap()
                        .error(format!("Error while accepting: {}", e));
                }
            } else {
                pending.reject();
            }
        }

        if let Some(res) = self
            .channel_aes_modal
            .as_mut()
            .and_then(|modal| modal.show(ctx))
        {
            let pending = self.channel_aes_modal.take().unwrap().inner().pending();
            if let Some(args) = res {
                if let Err(e) = self.app.add_aes_channel(pending, args.0, args.1, args.2) {
                    self.event_handler
                        .lock()
                        .unwrap()
                        .error(format!("Error while accepting: {}", e));
                }
            } else {
                pending.reject();
            }
        }

        self.event_handler.lock().unwrap().ui(ctx);
    }
}
