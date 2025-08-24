mod message;
pub use message::Message;

mod packet;
pub use packet::{FromPacket, IntoPacket, Packet};

mod rsa_handshake;
pub use rsa_handshake::RsaHandshake;

mod aes_handshake;
pub use aes_handshake::AesHandshake;

pub const RSA_KEY_SIZE: u32 = 4096;
pub const AES_KEY_SIZE: usize = 256 / 8;
