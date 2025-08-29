use std::io;

use egui::Ui;

use grapevine_lib::PendingRsaHandshake;

use super::modal::Form;

pub struct ChannelAcceptRsaForm {
    pending: PendingRsaHandshake,
}

impl ChannelAcceptRsaForm {
    pub fn new(pending: PendingRsaHandshake) -> Self {
        ChannelAcceptRsaForm { pending }
    }

    pub fn pending(self) -> PendingRsaHandshake {
        self.pending
    }
}

impl Form<'_> for ChannelAcceptRsaForm {
    type Error = io::Error;
    type Ret = bool;

    fn show(&mut self, ui: &mut Ui) -> Result<Option<Self::Ret>, Self::Error> {
        ui.label(format!(
            "{} wants to exchange RSA keys. Do you wish to accept their public key?",
            self.pending.name()
        ));

        Ok(ui
            .horizontal(|ui| {
                if ui.button("Accept").clicked() {
                    Some(true)
                } else if ui.button("Reject").clicked() {
                    Some(false)
                } else {
                    None
                }
            })
            .inner)
    }
}
