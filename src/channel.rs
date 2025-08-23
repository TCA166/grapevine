use std::{
    net::TcpStream,
    ops::Deref,
    sync::{Arc, Mutex},
};

use openssl::pkey::{PKey, Private, Public};

use super::protocol::{FromPacket, Handshake, IntoPacket, Message, Packet};

pub struct Channel {
    stream: Mutex<TcpStream>,
    name: Option<String>,
    messages: Mutex<Vec<Message>>,
    our_private_key: PKey<Private>,
    their_public_key: PKey<Public>,
    listeners: Vec<Arc<Mutex<dyn MessageListener>>>,
}

impl Channel {
    pub fn new(mut stream: TcpStream) -> Self {
        // ok so first we generate a new private key for us
        let private_key = PKey::generate_ed25519().unwrap();

        // then we send it to the other party
        let our_handshake: Packet = Handshake::new(&private_key).into_packet(&private_key);
        our_handshake.to_writer(&mut stream).unwrap();

        // then we receive the other party's handshake
        let their_handshake_packet = Packet::from_reader(&mut stream).unwrap();
        let their_handshake: Handshake = Handshake::from_packet(&their_handshake_packet);
        // and get the public key from that handshake
        let their_public_key = their_handshake.public_key();

        // might as well verify the signature so that we know that the key they have sent is valid
        if !their_handshake_packet.verify(&their_public_key) {
            panic!("Invalid signature");
        }

        Self::with_keys(stream, private_key, their_public_key)
    }

    pub fn with_keys(stream: TcpStream, our_key: PKey<Private>, their_key: PKey<Public>) -> Self {
        Self {
            stream: Mutex::new(stream),
            name: None,
            messages: Mutex::new(Vec::new()),
            our_private_key: our_key,
            their_public_key: their_key,
            listeners: Vec::new(),
        }
    }

    fn receive(&self) -> Option<Packet> {
        if let Some(packet) = Packet::from_reader(&mut self.stream.lock().unwrap().deref()) {
            if !packet.verify(&self.their_public_key) {
                panic!("Invalid signature");
            }
            return Some(packet);
        }
        None
    }

    pub fn listen(&self) {
        while let Some(packet) = self.receive() {
            let message: Message = Message::from_packet(&packet);
            for listener in &self.listeners {
                listener.lock().unwrap().on_message(&message, self);
            }
            self.messages.lock().unwrap().push(message);
        }
    }

    pub fn send_message(&self, message: Message) {
        let packet = message.into_packet(&self.our_private_key);
        packet
            .to_writer(&mut self.stream.lock().unwrap().deref())
            .unwrap();
    }

    pub fn name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        } else {
            self.stream.lock().unwrap().peer_addr().unwrap().to_string()
        }
    }
}

pub trait MessageListener: Send {
    fn on_message(&mut self, message: &Message, channel: &Channel);
}
