use std::{
    io,
    net::{Ipv4Addr, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use log::debug;

use super::{
    channel::{Channel, ProtocolError},
    listener::{PendingConnection, listener_thread},
};

/// Convenience function that properly initializes the channel and starts listening
/// Meant to be used in a thread
fn add_channel(
    channels: Arc<Mutex<Vec<Arc<Channel>>>>,
    channel: Channel,
) -> Result<(), ProtocolError> {
    let channel = Arc::new(channel);
    channels.lock().unwrap().push(channel.clone());
    debug!("Starting listening thread for channel {}", channel.name());
    channel.listen()?;
    Ok(())
}

pub struct GrapevineApp {
    // core app
    channels: Arc<Mutex<Vec<Arc<Channel>>>>,
    pending_connections: Arc<Mutex<Vec<PendingConnection>>>,
    // listener threads
    server_thread: thread::JoinHandle<()>,
    channel_threads: Vec<thread::JoinHandle<Result<(), ProtocolError>>>,
}

impl GrapevineApp {
    pub fn new(address: Ipv4Addr, port: u16) -> Self {
        let pending_connections = Arc::new(Mutex::new(Vec::new()));

        let res = Self {
            channels: Arc::new(Mutex::new(Vec::new())),
            pending_connections: pending_connections.clone(),
            server_thread: thread::spawn(move || {
                debug!("Starting listener thread for {}:{}", address, port);
                listener_thread((address, port), pending_connections)
            }),
            channel_threads: Vec::new(),
        };

        return res;
    }

    /// Creates a new connection, in case of problems returns immediately.
    /// Based on that connection, creates a new channel in a thread,
    /// in case of rejection that thread will be terminated, but no error will
    /// be returned here
    pub fn new_channel(
        &mut self,
        ip: Ipv4Addr,
        port: u16,
        name: Option<String>,
    ) -> Result<(), io::Error> {
        let stream = TcpStream::connect((ip, port))?;

        let channels = self.channels.clone();
        self.channel_threads
            .push(thread::spawn(move || -> Result<(), ProtocolError> {
                match Channel::new(stream, name)? {
                    Some(channel) => add_channel(channels, channel),
                    None => Err(ProtocolError::VerificationError),
                }
            }));
        Ok(())
    }

    /// Adds a new channel to the application.
    pub fn add_channel(&mut self, channel: Channel) {
        let channels = self.channels.clone();
        self.channel_threads
            .push(thread::spawn(move || add_channel(channels, channel)));
    }

    pub fn channels(&self) -> &Arc<Mutex<Vec<Arc<Channel>>>> {
        &self.channels
    }

    pub fn inspect_pending(&mut self) -> Vec<PendingConnection> {
        self.pending_connections
            .lock()
            .unwrap()
            .drain(..)
            .collect::<Vec<_>>()
    }

    pub fn add_pending(&mut self, pending: PendingConnection) {
        self.pending_connections.lock().unwrap().push(pending);
    }
}

impl Default for GrapevineApp {
    fn default() -> Self {
        Self::new(Ipv4Addr::new(0, 0, 0, 0), 5000)
    }
}
