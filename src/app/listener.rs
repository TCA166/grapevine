use std::{
    io,
    net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs},
};

use super::{Shared, channel::Channel, events::HandleMessage};

pub struct PendingConnection {
    stream: TcpStream,
    name: String,
}

impl PendingConnection {
    pub fn accept(
        self,
        name: Option<String>,
        message_handler: Shared<dyn HandleMessage>,
    ) -> Result<Option<Channel>, io::Error> {
        Channel::new(self.stream, name, message_handler)
    }

    pub fn reject(self) {
        self.stream.shutdown(Shutdown::Both).unwrap();
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

/// 'Server' thread, that listens for incoming connections and creates new channels for each connection.
pub fn listener_thread<A: ToSocketAddrs>(addr: A, pending: Shared<Vec<PendingConnection>>) {
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
