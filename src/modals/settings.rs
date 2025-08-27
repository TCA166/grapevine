use std::{
    net::{AddrParseError, SocketAddr},
    ops::Not,
    str::FromStr,
};

use egui::{Frame, Ui};

use super::{super::settings::Settings, modal::Form};

#[derive(Default)]
pub struct SettingsForm {
    uname_input: String,
    server_active: bool,
    server_addr_input: String,
}

impl Form<'_> for SettingsForm {
    type Ret = Settings;
    type Error = AddrParseError;

    fn show(&mut self, ui: &mut Ui) -> Result<Option<Self::Ret>, Self::Error> {
        ui.label("Username");
        ui.text_edit_singleline(&mut self.uname_input);

        ui.label("Server");
        ui.checkbox(&mut self.server_active, "Enabled");
        ui.add_enabled_ui(self.server_active, |ui| {
            Frame::group(ui.style()).show(ui, |ui| {
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
    }
}

impl SettingsForm {
    pub fn new(settings_base: &Settings) -> Self {
        Self {
            uname_input: settings_base.username().to_string(),
            server_active: settings_base.listening().is_some(),
            server_addr_input: settings_base
                .listening()
                .and_then(|addr| Some(addr.to_string()))
                .unwrap_or(String::new()),
        }
    }
}
