use std::{
    io,
    net::{Ipv4Addr, TcpStream},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use super::{
    Shared,
    channel::{Channel, ProtocolError},
    events::{HandleChannelCreationError, HandleNewChannel, HandleThreadError},
    handler::{EventHandler, EventRecipient},
    listener::{PendingConnection, listener_thread},
};

type ChannelThreadResult = Result<(), (ProtocolError, Arc<Channel>)>;
type ChannelCreationThreadResult = Result<Arc<Channel>, ProtocolError>;

/// Convenience function that properly initializes the channel and starts listening
/// Meant to be used in a thread
fn add_channel(channels: Shared<Vec<Arc<Channel>>>, channel: Arc<Channel>) -> ChannelThreadResult {
    channels.lock().unwrap().push(channel.clone());

    if let Err(err) = channel.listen() {
        return Err((err, channel));
    } else {
        Ok(())
    }
}

fn watchdog(
    threads: Shared<Vec<JoinHandle<ChannelThreadResult>>>,
    creation_threads: Shared<Vec<JoinHandle<ChannelCreationThreadResult>>>,
    handler: Shared<EventHandler>,
) {
    loop {
        thread::sleep(std::time::Duration::from_millis(500));
        for thread in threads
            .lock()
            .unwrap()
            .extract_if(.., |thread| thread.is_finished())
        {
            if let Err((err, channel)) = thread.join().unwrap() {
                handler.lock().unwrap().on_thread_error(&err, &channel);
            }
        }

        for thread in creation_threads
            .lock()
            .unwrap()
            .extract_if(.., |thread| thread.is_finished())
        {
            match thread.join().unwrap() {
                Ok(channel) => handler.lock().unwrap().on_new_channel(&channel),
                Err(err) => handler.lock().unwrap().on_channel_creation_error(&err),
            }
        }
    }
}

pub struct GrapevineApp {
    // core app
    channels: Shared<Vec<Arc<Channel>>>,
    pending_connections: Shared<Vec<PendingConnection>>,
    // listener threads
    server_thread: JoinHandle<()>,
    channel_threads: Shared<Vec<JoinHandle<ChannelThreadResult>>>,
    channel_creation_threads: Shared<Vec<JoinHandle<ChannelCreationThreadResult>>>,
    watchdog_thread: JoinHandle<()>,
    handler: Shared<EventHandler>,
}

impl GrapevineApp {
    pub fn new(address: Ipv4Addr, port: u16) -> Self {
        let channels = Arc::new(Mutex::new(Vec::new()));
        let pending_connections = Arc::new(Mutex::new(Vec::new()));
        let channel_threads = Arc::new(Mutex::new(Vec::new()));
        let channel_creation_threads = Arc::new(Mutex::new(Vec::new()));

        let handler = Arc::new(Mutex::new(EventHandler::new(channels.clone())));

        let res = Self {
            channels: channels,
            pending_connections: pending_connections.clone(),
            server_thread: thread::spawn(move || {
                listener_thread((address, port), pending_connections)
            }),
            channel_threads: channel_threads.clone(),
            channel_creation_threads: channel_creation_threads.clone(),
            handler: handler.clone(),
            watchdog_thread: thread::spawn(move || {
                watchdog(channel_threads, channel_creation_threads, handler)
            }),
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
        let message_handler = self.handler.clone();
        let channel_threads = self.channel_threads.clone();

        self.channel_creation_threads
            .lock()
            .unwrap()
            .push(thread::spawn(
                move || -> Result<Arc<Channel>, ProtocolError> {
                    match Channel::new(stream, name, message_handler)? {
                        Some(channel) => {
                            let channel = Arc::new(channel);
                            let channel_copy = channel.clone();
                            channel_threads
                                .lock()
                                .unwrap()
                                .push(thread::spawn(move || add_channel(channels, channel_copy)));
                            Ok(channel)
                        }
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
        if let Some(channel) = pending.accept(name, self.handler.clone())? {
            let channels = self.channels.clone();
            let channel = Arc::new(channel);
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

    pub fn add_event_recipient(&mut self, recipient: Shared<dyn EventRecipient>) {
        self.handler.lock().unwrap().add_recipient(recipient);
    }
}
