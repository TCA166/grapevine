use std::sync::Arc;

use super::{
    channel::{Channel, ProtocolError},
    protocol::Message,
};

/// Can handle new messages
pub trait HandleMessage: Send {
    fn on_message(&mut self, message: &Message, channel: &Channel);
}

/// Can handle thread errors
pub trait HandleThreadError: Send {
    fn on_thread_error(&mut self, error: &ProtocolError, channel: &Arc<Channel>);
}

/// Can handle new channels
pub trait HandleNewChannel: Send {
    fn on_new_channel(&mut self, channel: &Arc<Channel>);
}

/// Can handle errors in threads that await to be accepted
pub trait HandleChannelCreationError: Send {
    fn on_channel_creation_error(&mut self, error: &ProtocolError);
}
