use std::net::Ipv4Addr;

use super::message::Message;

pub struct Channel {
    ip: Ipv4Addr,
    name: Option<String>,
    messages: Vec<Message>,
}

impl Channel {
    pub fn new(ip: Ipv4Addr) -> Self {
        Self {
            ip,
            name: None,
            messages: Vec::new(),
        }
    }
}
