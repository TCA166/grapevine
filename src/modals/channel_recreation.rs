use std::{
    net::{AddrParseError, SocketAddr},
    str::FromStr,
};

use egui::Ui;

use grapevine_lib::ChannelDesc;

use super::modal::Form;

pub struct ChannelRecreationForm {
    desc: ChannelDesc,
    channel_addr_input: String,
}

impl ChannelRecreationForm {
    pub fn new(desc: ChannelDesc) -> Self {
        Self {
            channel_addr_input: String::new(),
            desc,
        }
    }

    pub fn desc(self) -> ChannelDesc {
        self.desc
    }
}

impl<'a> Form<'a> for ChannelRecreationForm {
    type Ret = Option<SocketAddr>;
    type Error = AddrParseError;

    fn show(&mut self, ui: &mut Ui) -> Result<Option<Self::Ret>, Self::Error> {
        ui.label("Address");
        ui.text_edit_singleline(&mut self.channel_addr_input);

        ui.horizontal(|ui| {
            if ui.button("Create").clicked() {
                let addr = SocketAddr::from_str(&self.channel_addr_input)?;

                Ok(Some(Some(addr)))
            } else if ui.button("Cancel").clicked() {
                Ok(Some(None))
            } else {
                Ok(None)
            }
        })
        .inner
    }
}
