use std::{fs, io, mem, path::PathBuf};

use egui::Ui;
use openssl::pkey::{PKey, Private, Public};

use super::{
    super::{app::PendingAesHandshake, file_picker::FilePathInput},
    modal::Form,
};

pub struct ChannelAcceptAesForm {
    pending: PendingAesHandshake,
    name_input: String,
    public_key: String,
    private_key: String,
    default_path: PathBuf,
}

impl ChannelAcceptAesForm {
    pub fn new(pending: PendingAesHandshake, default_key_path: &PathBuf) -> Self {
        let default_key_path_str = default_key_path.to_string_lossy().to_string();
        Self {
            pending,
            name_input: String::new(),
            public_key: default_key_path_str.clone(),
            private_key: default_key_path_str,
            default_path: default_key_path.clone(),
        }
    }

    pub fn pending(self) -> PendingAesHandshake {
        self.pending
    }
}

impl Form<'_> for ChannelAcceptAesForm {
    type Ret = Option<(Option<String>, PKey<Private>, PKey<Public>)>;
    type Error = io::Error;

    fn show(&mut self, ui: &mut Ui) -> Result<Option<Self::Ret>, Self::Error> {
        ui.label(format!(
            "{} knows our public key, and expects us to know theirs",
            self.pending.name()
        ));

        ui.label("Given name");
        ui.text_edit_singleline(&mut self.name_input);

        ui.add(FilePathInput::new(
            &mut self.private_key,
            "Our private key path",
            &self.default_path,
        ));

        ui.add(FilePathInput::new(
            &mut self.public_key,
            "Their public key path",
            &self.default_path,
        ));

        ui.horizontal(|ui| {
            if ui.button("Connect").clicked() {
                let name = if self.name_input.is_empty() {
                    None
                } else {
                    Some(mem::take(&mut self.name_input))
                };

                let private_key = fs::read(&self.private_key)?;
                let public_key = fs::read(&self.public_key)?;

                Ok(Some(Some((
                    name,
                    PKey::private_key_from_pem(&private_key)?,
                    PKey::public_key_from_pem(&public_key)?,
                ))))
            } else if ui.button("Cancel").clicked() {
                Ok(Some(None))
            } else {
                Ok(None)
            }
        })
        .inner
    }
}
