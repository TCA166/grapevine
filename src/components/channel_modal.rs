use std::{
    error, io,
    net::{AddrParseError, SocketAddr},
    num::ParseIntError,
    str::FromStr,
};

use derive_more::{Display, From};
use egui::{Context, Modal};
use egui_notify::Toasts;

use super::super::app::{GrapevineApp, ProtocolError};

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

pub struct ChannelModal {
    channel_name_input: String,
    channel_addr_input: String,
    toasts: Toasts,
}

impl ChannelModal {
    pub fn new() -> Self {
        Self {
            channel_addr_input: String::new(),
            channel_name_input: String::new(),
            toasts: Toasts::default(),
        }
    }

    pub fn show(&mut self, ctx: &Context, app: &mut GrapevineApp) -> Option<()> {
        self.toasts.show(ctx);

        Modal::new("New channel".into())
            .show(ctx, |ui| -> Result<Option<()>, ChannelFormError> {
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
            })
            .inner
            .unwrap_or_else(|err| {
                self.toasts.error(format!("{}", &err));
                None
            })
    }
}
