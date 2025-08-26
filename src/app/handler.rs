use std::{ops::DerefMut, sync::Arc};

use super::{
    super::protocol::Message,
    Shared,
    channel::{Channel, ProtocolError},
    events::*,
};

pub struct EventHandler {
    channels: Shared<Vec<Arc<Channel>>>,
    recipients: Vec<Shared<dyn EventRecipient>>,
}

pub trait EventRecipient: Send {
    fn info(&mut self, message: &str);

    fn warn(&mut self, message: &str);

    fn error(&mut self, message: &str);

    fn success(&mut self, message: &str);
}

impl EventHandler {
    pub fn new(channels: Shared<Vec<Arc<Channel>>>) -> Self {
        Self {
            channels,
            recipients: Vec::new(),
        }
    }

    pub fn add_recipient(&mut self, recipient: Shared<dyn EventRecipient>) {
        self.recipients.push(recipient);
    }

    fn recipient_invoke(&mut self, f: impl Fn(&mut dyn EventRecipient) -> ()) {
        for recipient in &self.recipients {
            f(recipient.lock().unwrap().deref_mut())
        }
    }
}

impl EventRecipient for EventHandler {
    fn info(&mut self, message: &str) {
        self.recipient_invoke(|recipient| recipient.info(message));
    }

    fn warn(&mut self, message: &str) {
        self.recipient_invoke(|recipient| recipient.warn(message));
    }

    fn error(&mut self, message: &str) {
        self.recipient_invoke(|recipient| recipient.error(message));
    }

    fn success(&mut self, message: &str) {
        self.recipient_invoke(|recipient| recipient.success(message));
    }
}

impl HandleMessage for EventHandler {
    fn on_message(&mut self, _message: &Message, channel: &Channel) {
        self.info(&format!("Received message on {}", channel.name()))
    }
}

impl HandleThreadError for EventHandler {
    fn on_thread_error(&mut self, error: &ProtocolError, channel: &Arc<Channel>) {
        self.channels.lock().unwrap().retain(|c| c != channel);
        self.error(&format!("Thread error on {}: {}", channel.name(), error))
    }
}

impl HandleChannelCreationError for EventHandler {
    fn on_channel_creation_error(&mut self, error: &ProtocolError) {
        self.warn(&format!("Failed to create thread: {}", error))
    }
}

impl HandleNewChannel for EventHandler {
    fn on_new_channel(&mut self, channel: &Arc<Channel>) {
        self.success(&format!("New channel: {}", channel.name()));
    }
}
