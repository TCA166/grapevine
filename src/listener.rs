use std::{
    net::{TcpListener, ToSocketAddrs},
    sync::{Arc, Mutex},
    thread,
};

use log::warn;

use super::channel::Channel;

/// 'Server' thread, that listens for incoming connections and creates new channels for each connection.
pub fn listener_thread<A: ToSocketAddrs>(addr: A, channels: Arc<Mutex<Vec<Arc<Channel>>>>) {
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        if let Some(channel) = Channel::new(stream, None) {
            let channel = Arc::new(channel);
            channels.lock().unwrap().push(channel.clone());
            thread::spawn(move || channel.listen());
        } else {
            warn!("Failed to accept connection");
        }
    }
}
