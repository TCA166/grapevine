use std::{error, io, net::TcpStream, ops::Deref, sync::Mutex};

use derive_more::{Display, From};
use log::{debug, info, warn};
use openssl::pkey::{PKey, Private, Public};

use super::protocol::{FromPacket, Handshake, IntoPacket, Message, Packet};

#[derive(Debug, Display, From)]
pub enum ProtocolError {
    IoError(io::Error),
    VerificationError,
}

impl error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

pub struct Channel {
    stream: Mutex<TcpStream>,
    name: String,
    messages: Mutex<Vec<Message>>,
    our_private_key: PKey<Private>,
    their_public_key: PKey<Public>,
}

impl Channel {
    pub fn new(mut stream: TcpStream, name: Option<String>) -> Option<Self> {
        // ok so first we generate a new private key for us
        let private_key = PKey::generate_ed25519().unwrap();
        info!("Generated new private key");

        // then we send it to the other party
        let our_handshake: Packet = Handshake::new(&private_key).into_packet(&private_key);
        our_handshake.to_writer(&mut stream).unwrap();
        debug!("Sent our handshake");

        // then we receive the other party's handshake
        let their_handshake_packet = Packet::from_reader(&mut stream).unwrap();
        let their_handshake: Handshake = Handshake::from_packet(&their_handshake_packet);
        // and get the public key from that handshake
        let their_public_key = their_handshake.public_key();
        info!("Received their public key");

        // might as well verify the signature so that we know that the key they have sent is valid
        if !their_handshake_packet.verify(&their_public_key) {
            warn!("Their public key is invalid");
            None
        } else {
            Some(Self::with_keys(stream, private_key, their_public_key, name))
        }
    }

    pub fn with_keys(
        stream: TcpStream,
        our_key: PKey<Private>,
        their_key: PKey<Public>,
        name: Option<String>,
    ) -> Self {
        let name = name.unwrap_or(stream.peer_addr().unwrap().to_string());
        info!("Creating new channel with name: {}", name);

        Self {
            stream: Mutex::new(stream),
            name: name,
            messages: Mutex::new(Vec::new()),
            our_private_key: our_key,
            their_public_key: their_key,
        }
    }

    pub fn listen(&self) -> Result<(), ProtocolError> {
        let mut stream = self.stream.lock().unwrap().try_clone()?;
        loop {
            let packet = Packet::from_reader(&mut stream)?;
            if !packet.verify(&self.their_public_key) {
                warn!("Received packet with invalid signature");
                return Err(ProtocolError::VerificationError);
            }
            let message: Message = Message::from_packet(&packet);
            debug!("Received message");
            self.messages.lock().unwrap().push(message);
        }
    }

    pub fn send_message(&self, message: Message) -> Result<(), ProtocolError> {
        let packet = message.into_packet(&self.our_private_key);
        self.messages.lock().unwrap().push(message);
        packet.to_writer(&mut self.stream.lock().unwrap().deref())?;
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn messages(&self) -> &Mutex<Vec<Message>> {
        &self.messages
    }
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
