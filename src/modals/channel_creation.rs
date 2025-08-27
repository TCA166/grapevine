use std::{
    error,
    net::{AddrParseError, SocketAddr},
    num::ParseIntError,
    str::FromStr,
};

use derive_more::{Display, From};

use super::modal::Form;

#[derive(Debug, From, Display)]
pub enum ChannelFormError {
    InvalidPort(ParseIntError),
    InvalidIp(AddrParseError),
}

impl error::Error for ChannelFormError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidIp(e) => Some(e),
            Self::InvalidPort(e) => Some(e),
        }
    }
}

#[derive(Default)]
pub struct ChannelForm {
    channel_name_input: String,
    channel_addr_input: String,
}

impl<'a> Form<'a> for ChannelForm {
    type Ret = Option<(SocketAddr, Option<String>)>;
    type Error = ChannelFormError;

    fn show(&mut self, ui: &mut egui::Ui) -> Result<Option<Self::Ret>, Self::Error> {
        ui.label("Channel Name");
        ui.text_edit_singleline(&mut self.channel_name_input);

        ui.label("Address");
        ui.text_edit_singleline(&mut self.channel_addr_input);

        ui.horizontal(|ui| {
            if ui.button("Create").clicked() {
                let addr = SocketAddr::from_str(&self.channel_addr_input)?;
                let name = Some(self.channel_name_input.clone()).filter(|s| !s.is_empty());

                Ok(Some(Some((addr, name))))
            } else if ui.button("Cancel").clicked() {
                Ok(Some(None))
            } else {
                Ok(None)
            }
        })
        .inner
    }
}
