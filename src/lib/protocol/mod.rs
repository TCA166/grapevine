/// Low level serialization for [RsaHandshake], [AesHandshake], [Message]
mod packet;
pub use packet::{FromPacket, IntoPacket, Packet};

/// Basic connection initialization
mod handshake;
pub use handshake::{Handshake, ProtocolPath};

mod rsa_handshake;
pub use rsa_handshake::RsaHandshake;

mod aes_handshake;
pub use aes_handshake::AesHandshake;

mod message;
pub use message::Message;

/// Routines for serializing buffers.
///
/// We serialize buffers by prepending them with VarInt encoded length, followed
/// with the buffer contents.
mod io;

const AES_KEY_SIZE: usize = 256 / 8;
const AES_IV_SIZE: usize = 128 / 8;

pub type AesKey = [u8; AES_KEY_SIZE];
pub type AesIv = [u8; AES_IV_SIZE];

use openssl::{error::ErrorStack, rand::rand_bytes};

pub fn new_aes_key() -> Result<AesKey, ErrorStack> {
    let mut aes_key = [0; AES_KEY_SIZE];
    rand_bytes(&mut aes_key)?;
    return Ok(aes_key);
}

pub fn new_aes_iv() -> Result<AesIv, ErrorStack> {
    let mut iv = [0; AES_IV_SIZE];
    rand_bytes(&mut iv)?;
    return Ok(iv);
}
