use std::{
    net::{AddrParseError, SocketAddr},
    ops::Not,
    str::FromStr,
};

use egui::{Context, Frame, Modal, Style};
use egui_notify::Toasts;

use super::super::settings::Settings;

pub struct SettingsModal {
    uname_input: String,
    server_active: bool,
    server_addr_input: String,
    toasts: Toasts,
}

impl SettingsModal {
    pub fn new(settings_base: &Settings) -> Self {
        Self {
            uname_input: settings_base.username().to_string(),
            server_active: settings_base.listening().is_some(),
            server_addr_input: settings_base
                .listening()
                .and_then(|addr| Some(addr.to_string()))
                .unwrap_or(String::new()),
            toasts: Toasts::default(),
        }
    }
}

impl SettingsModal {
    pub fn show(&mut self, ctx: &Context) -> Option<Settings> {
        self.toasts.show(ctx);

        Modal::new("Settings".into())
            .show(ctx, |ui| -> Result<Option<Settings>, AddrParseError> {
                ui.label("Username");
                ui.text_edit_singleline(&mut self.uname_input);

                ui.label("Server");
                ui.checkbox(&mut self.server_active, "Enabled");
                ui.add_enabled_ui(self.server_active, |ui| {
                    Frame::group(&Style::default()).show(ui, |ui| {
                        ui.text_edit_singleline(&mut self.server_addr_input);
                    });
                });

                if ui.button("Save").clicked() {
                    return Ok(Some(Settings::new(
                        self.server_active
                            .then(|| SocketAddr::from_str(&self.server_addr_input))
                            .transpose()?,
                        self.uname_input
                            .is_empty()
                            .not()
                            .then_some(self.uname_input.clone()),
                    )));
                } else {
                    return Ok(None);
                }
            })
            .inner
            .unwrap_or_else(|err| {
                self.toasts.error(format!("{}", &err));
                None
            })
    }
}
