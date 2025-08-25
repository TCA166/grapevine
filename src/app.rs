use std::{
    io,
    net::{Ipv4Addr, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use super::{
    channel::{Channel, ProtocolError},
    events::HandleOnMessage,
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
    channel.listen()?;
    Ok(())
}

fn watchdog(threads: Arc<Mutex<Vec<thread::JoinHandle<Result<(), ProtocolError>>>>>) {
    loop {
        thread::sleep(std::time::Duration::from_millis(500));
        for thread in threads
            .lock()
            .unwrap()
            .extract_if(.., |thread| thread.is_finished())
        {
            if let Err(err) = thread.join().unwrap() {}
        }
    }
}

pub struct GrapevineApp {
    // core app
    channels: Arc<Mutex<Vec<Arc<Channel>>>>,
    pending_connections: Arc<Mutex<Vec<PendingConnection>>>,
    // listener threads
    server_thread: thread::JoinHandle<()>,
    channel_threads: Arc<Mutex<Vec<thread::JoinHandle<Result<(), ProtocolError>>>>>,
    watchdog_thread: thread::JoinHandle<()>,
    message_handler: Option<Arc<Mutex<dyn HandleOnMessage>>>,
}

impl GrapevineApp {
    pub fn new(address: Ipv4Addr, port: u16) -> Self {
        let pending_connections = Arc::new(Mutex::new(Vec::new()));
        let channel_threads = Arc::new(Mutex::new(Vec::new()));

        let res = Self {
            channels: Arc::new(Mutex::new(Vec::new())),
            pending_connections: pending_connections.clone(),
            server_thread: thread::spawn(move || {
                listener_thread((address, port), pending_connections)
            }),
            channel_threads: channel_threads.clone(),
            watchdog_thread: thread::spawn(move || watchdog(channel_threads)),
            message_handler: None,
        };

        return res;
    }

    pub fn set_message_handler(&mut self, handler: Arc<Mutex<dyn HandleOnMessage>>) {
        self.message_handler = Some(handler);
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
        let message_handler = self.message_handler.clone();
        self.channel_threads.lock().unwrap().push(thread::spawn(
            move || -> Result<(), ProtocolError> {
                match Channel::new(stream, name, message_handler)? {
                    Some(channel) => add_channel(channels, channel),
                    None => Err(ProtocolError::VerificationError),
                }
            },
        ));
        Ok(())
    }

    /// Adds a new channel to the application.
    pub fn add_channel(
        &mut self,
        pending: PendingConnection,
        name: Option<String>,
    ) -> Result<(), ProtocolError> {
        if let Some(channel) = pending.accept(name, self.message_handler.clone())? {
            let channels = self.channels.clone();
            self.channel_threads
                .lock()
                .unwrap()
                .push(thread::spawn(move || add_channel(channels, channel)));
        }
        Ok(())
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
