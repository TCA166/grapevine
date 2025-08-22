use std::{net::TcpStream, ops::Deref, sync::Mutex};

use openssl::pkey::{PKey, Private, Public};

use super::protocol::{Handshake, Message, Packet};

pub struct Channel {
    stream: Mutex<TcpStream>,
    name: Option<String>,
    messages: Vec<Message>,
    our_private_key: PKey<Private>,
    their_public_key: PKey<Public>,
}

impl Channel {
    pub fn new(mut stream: TcpStream) -> Self {
        let private_key = PKey::generate_ed25519().unwrap();

        let our_handshake: Packet = Handshake::new(&private_key).into_packet(&private_key);
        our_handshake.to_writer(&mut stream).unwrap();

        let their_handshake: Handshake = Packet::from_reader(&mut stream).unwrap().into();

        Self::with_keys(stream, private_key, their_handshake.public_key())
    }

    pub fn with_keys(stream: TcpStream, our_key: PKey<Private>, their_key: PKey<Public>) -> Self {
        Self {
            stream: Mutex::new(stream),
            name: None,
            messages: Vec::new(),
            our_private_key: our_key,
            their_public_key: their_key,
        }
    }

    pub fn receive(&self) -> Option<Packet> {
        if let Some(packet) = Packet::from_reader(&mut self.stream.lock().unwrap().deref()) {
            if !packet.verify(&self.their_public_key) {
                panic!("Invalid signature");
            }
            return Some(packet);
        }
        None
    }
}
