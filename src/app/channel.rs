use std::{error, io, net::TcpStream, ops::Deref, sync::Mutex};

use derive_more::{Display, From};
use openssl::{
    pkey::{PKey, Private, Public},
    rand::rand_bytes,
    rsa::Rsa,
};

use super::{
    super::protocol::{
        AES_KEY_SIZE, AesHandshake, FromPacket, IntoPacket, Message, Packet, RSA_KEY_SIZE,
        RsaHandshake,
    },
    Shared,
    events::HandleMessage,
};

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

/// A channel for exchanging messages, through a specified stream
pub struct Channel {
    stream: Mutex<TcpStream>,
    name: String,
    messages: Mutex<Vec<Message>>,
    /// Our key for signing and AES key decryption
    our_rsa_private_key: PKey<Private>,
    /// Our key for encrypting messages
    our_aes_key: [u8; AES_KEY_SIZE],
    /// The key for checking the signature of messages, and encrypting the AES key
    their_rsa_public_key: PKey<Public>,
    /// The key for decrypting messages
    their_aes_key: [u8; AES_KEY_SIZE],
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
        mut stream: TcpStream,
        our_key: PKey<Private>,
        their_key: PKey<Public>,
        name: Option<String>,
        message_handler: Shared<dyn HandleMessage>,
    ) -> Result<Option<Self>, io::Error> {
        let mut our_aes_key = [0; AES_KEY_SIZE];
        rand_bytes(&mut our_aes_key)?;

        let our_aes_handshake = AesHandshake::new(&our_aes_key, &their_key)?;
        our_aes_handshake
            .into_packet(&our_key)
            .to_writer(&mut stream)?;

        let their_aes_handshake_packet = Packet::from_reader(&mut stream)?;
        if !their_aes_handshake_packet.verify(&their_key) {
            return Ok(None);
        }
        let their_aes_key = AesHandshake::from_packet(&their_aes_handshake_packet)
            .decrypt_key(&our_key)
            .unwrap();

        let name = name.unwrap_or(stream.peer_addr().unwrap().to_string());

        Ok(Some(Self {
            stream: Mutex::new(stream),
            name: name,
            messages: Mutex::new(Vec::new()),
            our_rsa_private_key: our_key,
            our_aes_key: our_aes_key,
            their_rsa_public_key: their_key,
            their_aes_key: their_aes_key,
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
            if !packet.verify(&self.their_rsa_public_key) {
                return Err(ProtocolError::VerificationError);
            }

            let message: Message = Message::from_packet(&packet);

            self.message_handler
                .lock()
                .unwrap()
                .on_message(&message, self);

            self.messages.lock().unwrap().push(message);
        }
    }

    /// Send a message to the channel
    pub fn send_message(&self, message: Message) -> Result<(), ProtocolError> {
        let mut packet = message.into_packet(&self.our_rsa_private_key);
        packet.encrypt(&self.their_aes_key)?;
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
