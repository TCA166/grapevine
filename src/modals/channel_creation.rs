use std::{
    error, fs, io,
    net::{AddrParseError, SocketAddr},
    str::FromStr,
};

use derive_more::{Display, From};
use egui::{Frame, Ui};
use openssl::{
    error::ErrorStack,
    pkey::{PKey, Private, Public},
};

use super::{super::widgets::FilePathInput, CUR_PATH, modal::Form};

#[derive(Debug, From, Display)]
pub enum ChannelFormError {
    InvalidIp(AddrParseError),
    IoError(io::Error),
    OpenSSL(ErrorStack),
}

impl error::Error for ChannelFormError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidIp(e) => Some(e),
            Self::IoError(e) => Some(e),
            Self::OpenSSL(e) => Some(e),
        }
    }
}

pub enum ChannelArgs {
    Rsa((SocketAddr, Option<String>)),
    Aes((SocketAddr, Option<String>, PKey<Private>, PKey<Public>)),
}

#[derive(Default)]
pub struct ChannelForm {
    channel_name_input: String,
    channel_addr_input: String,
    aes_skip: bool,
    public_key_path: String,
    private_key_path: String,
}

impl<'a> Form<'a> for ChannelForm {
    type Ret = Option<ChannelArgs>;
    type Error = ChannelFormError;

    fn show(&mut self, ui: &mut Ui) -> Result<Option<Self::Ret>, Self::Error> {
        ui.label("Channel Name");
        ui.text_edit_singleline(&mut self.channel_name_input);

        ui.label("Address");
        ui.text_edit_singleline(&mut self.channel_addr_input);

        ui.checkbox(&mut self.aes_skip, "Known keys");
        ui.add_enabled_ui(self.aes_skip, |ui| {
            Frame::group(ui.style()).show(ui, |ui| {
                ui.add(FilePathInput::new(
                    &mut self.private_key_path,
                    "Our private key path",
                    &CUR_PATH,
                ));

                ui.add(FilePathInput::new(
                    &mut self.public_key_path,
                    "Recipient public key path",
                    &CUR_PATH,
                ));
            })
        });

        ui.horizontal(|ui| {
            if ui.button("Create").clicked() {
                let addr = SocketAddr::from_str(&self.channel_addr_input)?;
                let name = Some(self.channel_name_input.clone()).filter(|s| !s.is_empty());

                Ok(Some(Some(match self.aes_skip {
                    false => ChannelArgs::Rsa((addr, name)),
                    true => ChannelArgs::Aes((
                        addr,
                        name,
                        PKey::private_key_from_pem(&fs::read(&self.private_key_path)?)?,
                        PKey::public_key_from_pem(&fs::read(&self.public_key_path)?)?,
                    )),
                })))
            } else if ui.button("Cancel").clicked() {
                Ok(Some(None))
            } else {
                Ok(None)
            }
        })
        .inner
    }
}
