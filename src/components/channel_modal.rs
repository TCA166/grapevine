use std::{
    error, io,
    net::{AddrParseError, SocketAddr},
    num::ParseIntError,
    str::FromStr,
};

use derive_more::{Display, From};

use super::{
    super::app::{GrapevineApp, ProtocolError},
    modal::Form,
};

#[derive(Debug, From, Display)]
pub enum ChannelFormError {
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

#[derive(Default)]
pub struct ChannelForm {
    channel_name_input: String,
    channel_addr_input: String,
}

impl<'a> Form<'a> for ChannelForm {
    type Args = &'a mut GrapevineApp;
    type Ret = ();
    type Error = ChannelFormError;

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        app: Self::Args,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        ui.label("Channel Name");
        ui.text_edit_singleline(&mut self.channel_name_input);

        ui.label("Address");
        ui.text_edit_singleline(&mut self.channel_addr_input);

        ui.horizontal(|ui| {
            if ui.button("Create").clicked() {
                let addr = SocketAddr::from_str(&self.channel_addr_input)?;
                let name = Some(self.channel_name_input.clone()).filter(|s| !s.is_empty());

                app.new_channel(addr, name)?;

                Ok(Some(()))
            } else if ui.button("Cancel").clicked() {
                Ok(Some(()))
            } else {
                Ok(None)
            }
        })
        .inner
    }
}
