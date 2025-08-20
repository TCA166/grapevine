use std::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::Arc,
};

use super::channel::Channel;

fn stream_handler(stream: TcpStream, channel: Arc<Channel>) {}

pub fn listener_thread<A: ToSocketAddrs>(addr: A, channels: Arc<Vec<Arc<Channel>>>) {
    let listener = TcpListener::bind(addr).unwrap();
    let mut connections = Vec::new();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
    }
}
