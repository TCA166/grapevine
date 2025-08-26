use std::{
    io,
    net::{SocketAddr, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
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

/// App backend, with no reference to the UI.
/// In theory this could be used in a CLI no problem.
pub struct GrapevineApp {
    /// Active channels
    channels: Shared<Vec<Arc<Channel>>>,
    /// Incoming connections we aren't sure we want to accept
    pending_connections: Shared<Vec<PendingConnection>>,

    /// Thread responsible for listening to new incoming connections
    listening: Arc<AtomicBool>,
    server_thread: Option<JoinHandle<()>>,
    /// Threads listening for new messages
    channel_threads: Shared<Vec<JoinHandle<ChannelThreadResult>>>,
    /// Threads creating new channels
    channel_creation_threads: Shared<Vec<JoinHandle<ChannelCreationThreadResult>>>,
    /// Thread monitoring other threads
    watchdog_thread: JoinHandle<()>,

    /// Our internal event handler
    handler: Shared<EventHandler>,
}

impl GrapevineApp {
    /// Create a new app instance, starting a server thread listening
    /// on the given address and port
    pub fn new() -> Self {
        let channels = Arc::new(Mutex::new(Vec::new()));
        let channel_threads = Arc::new(Mutex::new(Vec::new()));
        let channel_creation_threads = Arc::new(Mutex::new(Vec::new()));

        let handler = Arc::new(Mutex::new(EventHandler::new(channels.clone())));

        Self {
            channels: channels,
            pending_connections: Arc::new(Mutex::new(Vec::new())),
            listening: Arc::new(AtomicBool::new(false)),
            server_thread: None,
            channel_threads: channel_threads.clone(),
            channel_creation_threads: channel_creation_threads.clone(),
            handler: handler.clone(),
            watchdog_thread: thread::spawn(move || {
                watchdog(channel_threads, channel_creation_threads, handler)
            }),
        }
    }

    /// Creates a new connection, in case of problems returns immediately.
    /// Based on that connection, creates a new [Channel] in a thread,
    /// in case of rejection that thread will be terminated, but no error will
    /// be returned here
    pub fn new_channel(&mut self, addr: SocketAddr, name: Option<String>) -> Result<(), io::Error> {
        let stream = TcpStream::connect(addr)?;

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

    /// Accepts a [PendingConnection], and adds it as a [Channel] to the app
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

    /// Gets the list of currently ongoing channels
    pub fn channels(&self) -> &Arc<Mutex<Vec<Arc<Channel>>>> {
        &self.channels
    }

    /// Clears and returns the list of currently pending connections
    pub fn inspect_pending(&mut self) -> Vec<PendingConnection> {
        self.pending_connections
            .lock()
            .unwrap()
            .drain(..)
            .collect::<Vec<_>>()
    }

    /// Adds a new pending connection
    pub fn add_pending(&mut self, pending: PendingConnection) {
        self.pending_connections.lock().unwrap().push(pending);
    }

    /// Adds a listener that will receive all app wide events
    pub fn add_event_recipient(&mut self, recipient: Shared<dyn EventRecipient>) {
        self.handler.lock().unwrap().add_recipient(recipient);
    }

    pub fn stop_listening(&mut self) {
        if let Some(server) = self.server_thread.take() {
            self.listening.store(false, Ordering::Relaxed);

            server.join().unwrap();
        }
    }

    pub fn start_listening(&mut self, addr: SocketAddr) {
        if self.listening.swap(true, Ordering::Relaxed) {
            self.stop_listening();
        }

        let pending = self.pending_connections.clone();
        let listening = self.listening.clone();
        self.server_thread = Some(thread::spawn(move || {
            listener_thread(addr, pending, listening)
        }))
    }
}
