use std::{
    error, io,
    net::{Shutdown, SocketAddr, TcpStream},
    ops::Deref,
    sync::Mutex,
};

use chrono::Utc;
use derive_more::{Display, From};
use openssl::{
    pkey::{PKey, Private, Public},
    rsa::Rsa,
};

use super::{
    Shared,
    events::HandleMessage,
    protocol::{
        AesHandshake, AesKey, FromPacket, IntoPacket, Message, Packet, RsaHandshake, new_aes_key,
    },
};

const RSA_KEY_SIZE: u32 = 2048;

/// An error that has occured during [Packet] exchange
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

#[derive(Clone)]
pub struct ChannelDesc {
    name: String,
    last_addr: SocketAddr,
    /// Our key for signing and AES key decryption
    our_rsa_private_key: PKey<Private>,
    /// The key for checking the signature of messages, and encrypting the AES key
    their_rsa_public_key: PKey<Public>,
}

impl ChannelDesc {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn last_addr(&self) -> &SocketAddr {
        &self.last_addr
    }

    pub fn rename(&mut self, new: String) {
        self.name = new;
    }

    pub fn change_addr(&mut self, addr: SocketAddr) {
        self.last_addr = addr;
    }
}

impl PartialEq for ChannelDesc {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.last_addr == other.last_addr
            && self
                .our_rsa_private_key
                .public_eq(&other.our_rsa_private_key)
            && self
                .their_rsa_public_key
                .public_eq(&other.their_rsa_public_key)
    }
}

/// A channel for exchanging messages, through a specified stream
pub struct Channel {
    stream: Mutex<TcpStream>,
    messages: Mutex<Vec<Message>>,
    desc: ChannelDesc,
    /// The key for decrypting messages
    their_aes_key: AesKey,
    /// Our key for encrypting messages
    our_aes_key: AesKey,
    /// An abstract listener for new messages
    message_handler: Shared<dyn HandleMessage>,
}

impl Channel {
    /// Create a new channel, with the given stream and name
    pub fn new(
        mut stream: TcpStream,
        name: Option<String>,
        message_handler: Shared<dyn HandleMessage>,
    ) -> Result<Option<Self>, io::Error> {
        // ok so first we generate a new private key for us
        let private_rsa_key = PKey::from_rsa(Rsa::generate(RSA_KEY_SIZE)?)?;

        // then we send it to the other party
        let our_handshake = RsaHandshake::new(&private_rsa_key).into_packet(&private_rsa_key);
        our_handshake.to_writer(&mut stream)?;

        // then we receive the other party's handshake
        let their_handshake_packet = Packet::from_reader(&mut stream)?;
        let their_handshake = RsaHandshake::from_packet(&their_handshake_packet);
        // and get the public key from that handshake
        let their_public_key = their_handshake.public_key();

        // might as well verify the signature so that we know that the key they have sent is valid
        if !their_handshake_packet.verify(&their_public_key) {
            Ok(None)
        } else {
            Self::with_keys(
                stream,
                private_rsa_key,
                their_public_key,
                name,
                message_handler,
            )
        }
    }

    /// Create a new channel, assuming the RSA handshake has already happened
    pub fn with_keys(
        stream: TcpStream,
        our_rsa_private_key: PKey<Private>,
        their_rsa_public_key: PKey<Public>,
        name: Option<String>,
        message_handler: Shared<dyn HandleMessage>,
    ) -> Result<Option<Self>, io::Error> {
        let last_addr = stream.peer_addr()?;
        let desc = ChannelDesc {
            name: name.unwrap_or(last_addr.to_string()),
            last_addr,
            our_rsa_private_key,
            their_rsa_public_key,
        };
        Self::from_desc(stream, desc, message_handler)
    }

    /// Create a new channel, utilizing a previously saved [ChannelDesc]
    pub fn from_desc(
        mut stream: TcpStream,
        desc: ChannelDesc,
        message_handler: Shared<dyn HandleMessage>,
    ) -> Result<Option<Self>, io::Error> {
        let our_aes_key = new_aes_key()?;

        let our_aes_handshake = AesHandshake::new(&our_aes_key, &desc.their_rsa_public_key)?;
        our_aes_handshake
            .into_packet(&desc.our_rsa_private_key)
            .to_writer(&mut stream)?;

        let their_aes_handshake_packet = Packet::from_reader(&mut stream)?;
        if !their_aes_handshake_packet.verify(&desc.their_rsa_public_key) {
            return Ok(None);
        }
        let their_aes_key = AesHandshake::from_packet(&their_aes_handshake_packet)
            .decrypt_key(&desc.our_rsa_private_key)
            .unwrap();

        Ok(Some(Self {
            stream: Mutex::new(stream),
            messages: Mutex::new(Vec::new()),
            desc,
            our_aes_key,
            their_aes_key,
            message_handler: message_handler,
        }))
    }

    /// Listen for incoming messages on the channel
    /// This function will continuously listen for incoming messages until an error occurs
    pub fn listen(&self) -> Result<(), ProtocolError> {
        let mut stream = self.stream.lock().unwrap().try_clone()?; // important to avoid deadlocks
        loop {
            let mut packet = Packet::from_reader(&mut stream)?;
            packet.decrypt(&self.our_aes_key)?;
            if !packet.verify(&self.desc.their_rsa_public_key) {
                return Err(ProtocolError::VerificationError);
            }

            let message: Message = Message::from_packet(&packet);

            if message.timestamp() > &Utc::now() {
                // if we received a message from the future
                continue;
            }

            if let Some(last_msg) = self.messages().lock().unwrap().last() {
                if last_msg.timestamp() > message.timestamp() {
                    // if we received a message from the past
                    continue;
                }
            }

            self.message_handler
                .lock()
                .unwrap()
                .on_message(&message, self);

            self.messages.lock().unwrap().push(message);
        }
    }

    /// Send a message to the channel
    pub fn send_message(&self, message: Message) -> Result<(), ProtocolError> {
        let mut packet = message.into_packet(&self.desc.our_rsa_private_key);
        packet.encrypt(&self.their_aes_key)?;
        self.messages.lock().unwrap().push(message);
        packet.to_writer(&mut self.stream.lock().unwrap().deref())?;
        Ok(())
    }

    /// Get the name of the channel
    pub fn name(&self) -> &str {
        self.desc.name()
    }

    /// Get the messages in the channel
    pub fn messages(&self) -> &Mutex<Vec<Message>> {
        &self.messages
    }

    /// Closes the channel
    pub fn close(&self) -> Result<(), io::Error> {
        self.stream.lock().unwrap().shutdown(Shutdown::Both)
    }

    pub fn desc(&self) -> &ChannelDesc {
        &self.desc
    }
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        self.desc.name == other.desc.name
    }
}
