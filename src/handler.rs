use std::ops::{Deref, DerefMut};

use egui::Context;
use egui_notify::Toasts;

use super::{
    channel::{Channel, ProtocolError},
    events::{HandleOnMessage, HandleThreadError},
    protocol::Message,
};

#[derive(Default)]
pub struct EventHandler {
    toasts: Toasts,
}

impl EventHandler {
    pub fn ui(&mut self, ctx: &Context) {
        self.toasts.show(ctx);
    }
}

impl Deref for EventHandler {
    type Target = Toasts;

    fn deref(&self) -> &Self::Target {
        &self.toasts
    }
}

impl DerefMut for EventHandler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.toasts
    }
}

impl HandleOnMessage for EventHandler {
    fn on_message(&mut self, _message: &Message, channel: &Channel) {
        self.toasts
            .info(format!("Received message from {}", channel.name()));
    }
}

impl HandleThreadError for EventHandler {
    fn on_thread_error(&mut self, error: &ProtocolError) {
        self.toasts
            .error(format!("Error while processing {}", error));
    }
}
