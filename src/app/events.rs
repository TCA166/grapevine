use std::sync::Arc;

use super::{
    super::protocol::Message,
    channel::{Channel, ProtocolError},
};

pub trait HandleMessage: Send {
    fn on_message(&mut self, message: &Message, channel: &Channel);
}

pub trait HandleThreadError: Send {
    fn on_thread_error(&mut self, error: &ProtocolError, channel: &Arc<Channel>);
}

pub trait HandleNewChannel: Send {
    fn on_new_channel(&mut self, channel: &Arc<Channel>);
}

pub trait HandleChannelCreationError: Send {
    fn on_channel_creation_error(&mut self, error: &ProtocolError);
}
