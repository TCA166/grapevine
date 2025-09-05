use std::{
    io,
    net::{Shutdown, SocketAddr, TcpStream},
    ops::Deref,
    sync::Mutex,
};

use chrono::Utc;
use derive_more::{Display, Error, From};
use openssl::{
    error::ErrorStack,
    pkey::{PKey, Private, Public},
    rsa::Rsa,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{
    Shared,
    events::HandleMessage,
    protocol::{
        AesHandshake, AesKey, FromPacket, IntoPacket, Message, Packet, RsaHandshake, new_aes_key,
    },
};

const RSA_KEY_SIZE: u32 = 2048;

/// An error that has occured during [Packet] exchange
#[derive(Debug, Display, From, Error)]
pub enum ProtocolError {
    IoError(io::Error),
    VerificationError,
    SerializationError(bitcode::Error),
    OpenSSLError(ErrorStack),
}

fn serialize_private_key<S: Serializer>(
    key: &PKey<Private>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::Error;

    match key.private_key_to_pem_pkcs8() {
        Ok(vec) => serializer.serialize_bytes(&vec),
        Err(e) => Err(S::Error::custom(format!(
            "Failed to serialize private key: {}",
            e
        ))),
    }
}

fn deserialize_private_key<'de, D: Deserializer<'de>>(de: D) -> Result<PKey<Private>, D::Error> {
    use serde::de::Error;

    let bytes = <Vec<u8>>::deserialize(de)?;
    PKey::private_key_from_pem(&bytes)
        .map_err(|e| D::Error::custom(format!("Failed to parse private key: {}", e)))
}

fn serialize_public_key<S: Serializer>(
    key: &PKey<Public>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::Error;

    match key.public_key_to_pem() {
        Ok(vec) => serializer.serialize_bytes(&vec),
        Err(e) => Err(S::Error::custom(format!(
            "Failed to serialize public key: {}",
            e
        ))),
    }
}

fn deserialize_public_key<'de, D: Deserializer<'de>>(de: D) -> Result<PKey<Public>, D::Error> {
    use serde::de::Error;

    let bytes = <Vec<u8>>::deserialize(de)?;
    PKey::public_key_from_pem(&bytes)
        .map_err(|e| D::Error::custom(format!("Failed to parse private key: {}", e)))
}

/// Core [Channel] data, from which we can recreate a channel
#[derive(Clone, Serialize, Deserialize)]
pub struct ChannelDesc {
    name: String,
    last_addr: SocketAddr,
    /// Our key for signing and AES key decryption
    #[serde(
        serialize_with = "serialize_private_key",
        deserialize_with = "deserialize_private_key"
    )]
    our_rsa_private_key: PKey<Private>,
    /// The key for checking the signature of messages, and encrypting the AES key
    #[serde(
        serialize_with = "serialize_public_key",
        deserialize_with = "deserialize_public_key"
    )]
    their_rsa_public_key: PKey<Public>,
}

impl ChannelDesc {
    /// Get the name of the channel
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get the address, which was last assigned to the channel
    pub fn last_addr(&self) -> &SocketAddr {
        &self.last_addr
    }

    /// Rename the channel
    pub fn rename(&mut self, new: String) {
        self.name = new;
    }

    /// Change the address of the channel
    pub fn change_addr(&mut self, addr: SocketAddr) {
        self.last_addr = addr;
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
    /// Create a new channel on the given stream with a certain name.
    /// First the RSA exchange (handshake) is performed, followed by the AES key exchange.
    ///
    /// ## Args
    ///
    /// - `stream`: The stream to use for communication
    /// - `name`: The name of the channel
    /// - `message_handler`: The handler for new messages, which will be notified when a new message is received
    ///
    /// ## Returns
    ///
    /// A new channel instance, or an error if the channel could not be created.
    /// This may mainly happen if the handshake fails. In case, the verification
    /// of the other party's public key fails, None is returned.
    pub fn new(
        mut stream: TcpStream,
        name: Option<String>,
        message_handler: Shared<dyn HandleMessage>,
    ) -> Result<Option<Self>, ProtocolError> {
        // ok so first we generate a new private key for us
        let private_rsa_key = PKey::from_rsa(Rsa::generate(RSA_KEY_SIZE)?)?;

        // then we send it to the other party
        let our_handshake = RsaHandshake::new(&private_rsa_key).into_packet(&private_rsa_key)?;
        our_handshake.to_writer(&mut stream)?;

        // then we receive the other party's handshake
        let their_handshake_packet = Packet::from_reader(&mut stream)?;
        let their_handshake = RsaHandshake::from_packet(&their_handshake_packet)?;
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

    /// Create a new channel, assuming the RSA handshake has already happened.
    ///
    /// ## Args
    ///
    /// - `stream`: The stream to use for communication
    /// - `our_rsa_private_key`: Our private RSA key
    /// - `their_rsa_public_key`: The public RSA key of the other party
    /// - `name`: The name of the channel
    /// - `message_handler`: The handler for new messages, which will be notified when a new message is received
    ///
    /// ## Returns
    ///
    /// A new channel, Err if the handshake failed, or None if the verification of the other party's messages failed.
    pub fn with_keys(
        stream: TcpStream,
        our_rsa_private_key: PKey<Private>,
        their_rsa_public_key: PKey<Public>,
        name: Option<String>,
        message_handler: Shared<dyn HandleMessage>,
    ) -> Result<Option<Self>, ProtocolError> {
        let last_addr = stream.peer_addr()?;
        let desc = ChannelDesc {
            name: name.unwrap_or(last_addr.to_string()),
            last_addr,
            our_rsa_private_key,
            their_rsa_public_key,
        };
        Self::from_desc(stream, desc, message_handler)
    }

    /// Create a new channel, utilizing a previously saved [ChannelDesc].
    ///
    /// ## Args
    ///
    /// - `stream`: The TCP stream to use for the channel.
    /// - `desc`: The channel description.
    /// - `message_handler`: The message handler to use for the channel.
    ///
    /// ## Returns
    ///
    /// A new channel, Err if the handshake fails, None if the verification fails.
    pub fn from_desc(
        mut stream: TcpStream,
        desc: ChannelDesc,
        message_handler: Shared<dyn HandleMessage>,
    ) -> Result<Option<Self>, ProtocolError> {
        let our_aes_key = new_aes_key()?;

        let our_aes_handshake = AesHandshake::new(&our_aes_key, &desc.their_rsa_public_key)?;
        our_aes_handshake
            .into_packet(&desc.our_rsa_private_key)?
            .to_writer(&mut stream)?;

        let their_aes_handshake_packet = Packet::from_reader(&mut stream)?;
        if !their_aes_handshake_packet.verify(&desc.their_rsa_public_key) {
            return Ok(None);
        }
        let their_aes_key = AesHandshake::from_packet(&their_aes_handshake_packet)?
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

    /// Listen for incoming messages on the channel.
    /// This function will continuously listen for incoming messages until an
    /// error occurs, so ideally it should be run in a separate thread.
    pub fn listen(&self) -> Result<(), ProtocolError> {
        let mut stream = self.stream.lock().unwrap().try_clone()?; // important to avoid deadlocks
        loop {
            let mut packet = Packet::from_reader(&mut stream)?;
            packet.decrypt(&self.our_aes_key)?;
            if !packet.verify(&self.desc.their_rsa_public_key) {
                return Err(ProtocolError::VerificationError);
            }

            let message = Message::from_packet(&packet)?;

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
        let mut packet = message.into_packet(&self.desc.our_rsa_private_key)?;
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

    /// Get the description of the channel
    pub fn desc(&self) -> &ChannelDesc {
        &self.desc
    }
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        self.desc.name == other.desc.name
    }
}
