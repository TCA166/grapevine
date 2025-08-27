use std::{
    io,
    net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use openssl::pkey::{PKey, Private, Public};

use super::{
    super::protocol::{Handshake, ProtocolPath},
    Shared,
    channel::Channel,
    events::HandleMessage,
};

struct PendingHandshake {
    stream: TcpStream,
    name: String,
}

impl PendingHandshake {
    pub fn reject(self) {
        self.stream.shutdown(Shutdown::Both).unwrap();
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

pub struct PendingAesHandshake {
    inner: PendingHandshake,
}

impl PendingAesHandshake {
    pub fn accept(
        self,
        name: Option<String>,
        our_key: PKey<Private>,
        their_key: PKey<Public>,
        message_handler: Shared<dyn HandleMessage>,
    ) -> Result<Option<Channel>, io::Error> {
        Channel::with_keys(self.inner.stream, our_key, their_key, name, message_handler)
    }

    pub fn reject(self) {
        self.inner.reject();
    }

    pub fn name(&self) -> &str {
        self.inner.name()
    }
}

/// A connection that has yet to be accepted
pub struct PendingRsaHandshake {
    inner: PendingHandshake,
}

impl PendingRsaHandshake {
    pub fn accept(
        self,
        name: Option<String>,
        message_handler: Shared<dyn HandleMessage>,
    ) -> Result<Option<Channel>, io::Error> {
        Channel::new(self.inner.stream, name, message_handler)
    }

    pub fn reject(self) {
        self.inner.reject()
    }

    pub fn name(&self) -> &str {
        self.inner.name()
    }
}

pub enum PendingConnection {
    Rsa(PendingRsaHandshake),
    Aes(PendingAesHandshake),
}

impl PendingConnection {
    pub fn name(&self) -> &str {
        match self {
            PendingConnection::Aes(p) => p.name(),
            PendingConnection::Rsa(p) => p.name(),
        }
    }

    pub fn reject(self) {
        match self {
            PendingConnection::Aes(p) => p.reject(),
            PendingConnection::Rsa(p) => p.reject(),
        }
    }
}

/// 'Server' thread, that listens for incoming connections and creates new channels for each connection.
pub fn listener_thread<A: ToSocketAddrs>(
    addr: A,
    pending: Shared<Vec<PendingConnection>>,
    listening: Arc<AtomicBool>,
) {
    let listener = TcpListener::bind(addr).unwrap();

    listener.set_nonblocking(true).unwrap();

    for stream in listener.incoming() {
        if !listening.load(Ordering::Relaxed) {
            break;
        }

        if let Ok(mut stream) = stream {
            let name = stream.peer_addr().unwrap().to_string();

            if let Ok(handshake) = Handshake::from_reader(&mut stream) {
                if !handshake.version_ok() {
                    continue;
                }

                let inner = PendingHandshake { stream, name };

                let conn = match handshake.next() {
                    ProtocolPath::AesExchange => {
                        PendingConnection::Aes(PendingAesHandshake { inner })
                    }
                    ProtocolPath::RsaExchange => {
                        PendingConnection::Rsa(PendingRsaHandshake { inner })
                    }
                };

                pending.lock().unwrap().push(conn);
            }
        } else {
            sleep(Duration::from_millis(100));
        }
    }
}
