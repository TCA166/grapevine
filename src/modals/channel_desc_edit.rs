use std::{
    net::{AddrParseError, SocketAddr},
    str::FromStr,
};

use egui::Ui;

use grapevine_lib::ChannelDesc;

use super::modal::Form;

pub struct ChannelDescEditForm {
    channel_name_input: String,
    addr_input: String,
    desc: ChannelDesc,
}

impl ChannelDescEditForm {
    pub fn new(desc: ChannelDesc) -> Self {
        Self {
            channel_name_input: desc.name().to_owned(),
            addr_input: desc.last_addr().to_string(),
            desc,
        }
    }

    pub fn desc(mut self) -> ChannelDesc {
        self.desc.rename(self.channel_name_input);
        self.desc
    }
}

impl<'a> Form<'a> for ChannelDescEditForm {
    type Ret = ();
    type Error = AddrParseError;

    fn show(&mut self, ui: &mut Ui) -> Result<Option<Self::Ret>, Self::Error> {
        ui.label("Name");
        ui.text_edit_singleline(&mut self.channel_name_input);

        ui.label("Address");
        ui.text_edit_singleline(&mut self.addr_input);

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                let addr = SocketAddr::from_str(&self.channel_name_input)?;

                self.desc.change_addr(addr);

                Ok(Some(()))
            } else {
                Ok(None)
            }
        })
        .inner
    }
}
