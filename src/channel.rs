use std::net::TcpStream;

use super::{message::Message, packet::Packet};

pub struct Channel {
    stream: TcpStream,
    name: Option<String>,
    messages: Vec<Message>,
}

impl Channel {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            name: None,
            messages: Vec::new(),
        }
    }

    pub fn receive(&mut self) -> Option<Packet> {
        return Packet::from_reader(&mut self.stream);
    }
}
