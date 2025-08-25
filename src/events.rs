use super::{
    channel::{Channel, ProtocolError},
    protocol::Message,
};

pub trait HandleOnMessage: Send {
    fn on_message(&mut self, message: &Message, channel: &Channel);
}

pub trait HandleThreadError: Send {
    fn on_thread_error(&mut self, error: &ProtocolError);
}
