use std::{
    net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs},
    sync::{Arc, Mutex},
};

use log::warn;

use super::channel::Channel;

pub struct PendingConnection {
    stream: TcpStream,
    name: String,
}

impl PendingConnection {
    pub fn accept(self, name: Option<String>) -> Option<Channel> {
        let channel = Channel::new(self.stream, name);
        if let Some(channel) = channel {
            Some(channel)
        } else {
            warn!("Failed to accept connection");
            None
        }
    }

    pub fn reject(self) {
        self.stream.shutdown(Shutdown::Both).unwrap();
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

/// 'Server' thread, that listens for incoming connections and creates new channels for each connection.
pub fn listener_thread<A: ToSocketAddrs>(addr: A, pending: Arc<Mutex<Vec<PendingConnection>>>) {
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let name = stream.peer_addr().unwrap().to_string();

            pending
                .lock()
                .unwrap()
                .push(PendingConnection { stream, name });
        }
    }
}
