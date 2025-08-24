mod message;
pub use message::Message;

mod packet;
pub use packet::{FromPacket, IntoPacket, Packet};

mod rsa_handshake;
pub use rsa_handshake::RsaHandshake;

mod aes_handshake;
pub use aes_handshake::AesHandshake;

pub const RSA_KEY_SIZE: u32 = 2048;
pub const AES_KEY_SIZE: usize = 256 / 8;
pub const AES_IV_SIZE: usize = 128 / 8;
