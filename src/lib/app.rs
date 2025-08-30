use std::{
    io,
    net::{SocketAddr, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use openssl::pkey::{PKey, Private, Public};

use super::{
    Shared,
    channel::{Channel, ChannelDesc, ProtocolError},
    events::{HandleChannelCreationError, HandleNewChannel, HandleThreadError},
    handler::{EventHandler, EventRecipient},
    listener::{PendingAesHandshake, PendingConnection, PendingRsaHandshake, listener_thread},
    protocol::{Handshake, ProtocolPath},
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

/// Thread that monitors other threads for failures, and forwards that
/// information to the [EventHandler]
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
    /// Create a new app instance
    ///
    /// Initializes the [EventHandler] and [Self::watchdog_thread].
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

    /// Creates a new connection, assuming the RSA handshake will happen next.
    ///
    /// ## Args
    ///
    /// - addr: the address to which the new [Channel] should connect to
    /// - name: the name to give the [Channel]
    ///
    /// ## Returns
    ///
    /// In case of immediate connection errors, it will return an error.
    /// The [Channel] creation will happen in a separate thread though,
    /// meaning immediately after calling, the [Channel] won't be added to
    /// [Self::channels].
    pub fn new_rsa_channel(
        &mut self,
        addr: SocketAddr,
        name: Option<String>,
    ) -> Result<(), io::Error> {
        self.new_channel(
            addr,
            ProtocolPath::RsaExchange,
            |stream, message_handler| Channel::new(stream, name, message_handler),
        )
    }

    /// Creates a new connection, assuming the AES handshake will happen next.
    /// For more details look at [Self::new_rsa_channel].
    ///
    /// ## Args
    ///
    /// - addr: The address the new channel should connect to
    /// - our_key: Our private key, the recipient should have the corresponding public key
    /// - their_key: Their public key, the recipient should have the corresponding private key
    /// - name: The name to give the channel
    pub fn new_aes_channel(
        &mut self,
        addr: SocketAddr,
        our_key: PKey<Private>,
        their_key: PKey<Public>,
        name: Option<String>,
    ) -> Result<(), io::Error> {
        self.new_channel(
            addr,
            ProtocolPath::AesExchange,
            |stream, message_handler| {
                Channel::with_keys(stream, our_key, their_key, name, message_handler)
            },
        )
    }

    /// Recreates a new connection, based on the [ChannelDesc]ription.
    /// Roughly equivalent to [Self::new_aes_channel], except uses a compact
    /// struct for argument passing.
    ///
    /// ## Args
    ///
    /// - addr: the address to connect to
    /// - desc: The channel description struct
    pub fn new_channel_from_desc(
        &mut self,
        addr: SocketAddr,
        desc: ChannelDesc,
    ) -> Result<(), io::Error> {
        self.new_channel(
            addr,
            ProtocolPath::AesExchange,
            |stream, message_handler| Channel::from_desc(stream, desc, message_handler),
        )
    }

    /// Helper method that estabilishes the connection and handles the threading.
    ///
    /// The connection will be created immediately, alongside the handshake.
    /// The channel will be created in a helper thread, so that it can wait
    /// for getting accepted on the other side ([Self::channel_creation_threads]).
    /// After finishing the handshakes, the channel listening will happen
    /// in a new thread ([Self::channel_threads])
    fn new_channel(
        &mut self,
        addr: SocketAddr,
        path: ProtocolPath,
        creator: impl 'static
        + Send
        + FnOnce(TcpStream, Shared<EventHandler>) -> Result<Option<Channel>, io::Error>,
    ) -> Result<(), io::Error> {
        let mut stream = TcpStream::connect(addr)?;

        let channels = self.channels.clone();
        let message_handler = self.handler.clone();
        let channel_threads = self.channel_threads.clone();

        let handshake = Handshake::new(path);
        handshake.to_writer(&mut stream)?;

        self.channel_creation_threads
            .lock()
            .unwrap()
            .push(thread::spawn(
                move || -> Result<Arc<Channel>, ProtocolError> {
                    match creator(stream, message_handler)? {
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

    /// Accepts a [PendingRsaHandshake], and adds it as a [Channel] to the app
    pub fn add_rsa_channel(
        &mut self,
        pending: PendingRsaHandshake,
        name: Option<String>,
    ) -> Result<(), ProtocolError> {
        if let Some(channel) = pending.accept(name, self.handler.clone())? {
            self.add_channel(channel);
        }
        Ok(())
    }

    /// Accepts a [PendingAesHandshake], and adds it as a [Channel] to the app
    pub fn add_aes_channel(
        &mut self,
        pending: PendingAesHandshake,
        name: Option<String>,
        our_key: PKey<Private>,
        their_key: PKey<Public>,
    ) -> Result<(), ProtocolError> {
        if let Some(channel) = pending.accept(name, our_key, their_key, self.handler.clone())? {
            self.add_channel(channel);
        }
        Ok(())
    }

    /// Internal method that handles all the necessary details behind adding a [Channel]
    fn add_channel(&mut self, channel: Channel) {
        let channels = self.channels.clone();
        let channel = Arc::new(channel);
        self.channel_threads
            .lock()
            .unwrap()
            .push(thread::spawn(move || add_channel(channels, channel)));
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

    /// Stops the server thread
    pub fn stop_listening(&mut self) {
        if let Some(server) = self.server_thread.take() {
            self.listening.store(false, Ordering::Relaxed);

            server.join().unwrap();
        }
    }

    /// Starts the server thread
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
