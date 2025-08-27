use std::io;

use egui::Ui;

use super::{super::app::PendingRsaHandshake, modal::Form};

pub struct ChannelAcceptRsaForm {
    pending: PendingRsaHandshake,
    name_input: String,
}

impl ChannelAcceptRsaForm {
    pub fn new(pending: PendingRsaHandshake) -> Self {
        ChannelAcceptRsaForm {
            pending,
            name_input: String::new(),
        }
    }

    pub fn pending(self) -> PendingRsaHandshake {
        self.pending
    }
}

impl Form<'_> for ChannelAcceptRsaForm {
    type Error = io::Error;
    type Ret = Option<Option<String>>;

    fn show(&mut self, ui: &mut Ui) -> Result<Option<Self::Ret>, Self::Error> {
        ui.label("Given name");
        ui.text_edit_singleline(&mut self.name_input);

        ui.label(format!(
            "{} wants to exchange RSA keys. Do you wish to accept their public key?",
            self.pending.name()
        ));

        Ok(ui
            .horizontal(|ui| {
                if ui.button("Accept").clicked() {
                    let name = if self.name_input.is_empty() {
                        None
                    } else {
                        Some(self.name_input.clone())
                    };

                    Some(Some(name))
                } else if ui.button("Reject").clicked() {
                    Some(None)
                } else {
                    None
                }
            })
            .inner)
    }
}
