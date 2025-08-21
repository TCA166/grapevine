use std::{
    net::{TcpListener, ToSocketAddrs},
    sync::{Arc, Mutex},
};

use super::{
    channel::Channel,
    protocol::{Handshake, Message, Packet},
};

fn stream_handler(channel: Arc<Mutex<Channel>>) {
    while let Some(packet) = channel.lock().unwrap().receive() {
        let message: Message = packet.into();
    }
}

pub fn listener_thread<A: ToSocketAddrs>(addr: A, channels: Arc<Mutex<Vec<Arc<Mutex<Channel>>>>>) {
    let listener = TcpListener::bind(addr).unwrap();
    let mut connection_threads = Vec::new();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let channel = Arc::new(Mutex::new(Channel::new(stream)));
        channels.lock().unwrap().push(channel.clone());
        connection_threads.push(std::thread::spawn(move || stream_handler(channel)));
    }
}
