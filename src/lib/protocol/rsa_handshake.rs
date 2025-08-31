use openssl::pkey::{PKey, Private, Public};
use serde::{Deserialize, Serialize};

/// A handshake that delivers the public key of one party
/// Intended to be the first thing sent, optionally of course in case the
/// parties have already exchanged public keys.
#[derive(Serialize, Deserialize)]
pub struct RsaHandshake {
    public_key: Vec<u8>,
}

impl RsaHandshake {
    /// Creates a new handshake, utilizing the given private key.
    pub fn new(private_key: &PKey<Private>) -> Self {
        Self {
            public_key: private_key.public_key_to_pem().unwrap(),
        }
    }

    /// Parse the public key from the handshake.
    pub fn public_key(&self) -> PKey<Public> {
        PKey::public_key_from_pem(&self.public_key).unwrap()
    }
}

#[cfg(test)]
mod tests {

    use super::super::packet::{FromPacket, IntoPacket, Packet};
    use super::*;
    use openssl::rsa::Rsa;
    use std::io::Cursor;

    #[test]
    fn test_handshake_new() {
        let private_key = PKey::generate_ed25519().unwrap();
        let handshake = RsaHandshake::new(&private_key);
        let public_key = handshake.public_key();
        assert!(public_key.public_eq(&private_key));
    }

    #[test]
    fn test_packet_conversion() {
        let private_key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
        let handshake = RsaHandshake::new(&private_key);
        let public_key = handshake.public_key();
        let packet = handshake.into_packet(&private_key).unwrap();
        let mut data = Vec::new();
        packet.to_writer(&mut data).unwrap();
        let decoded_packet = Packet::from_reader(&mut Cursor::new(data)).unwrap();
        assert!(decoded_packet.verify(&public_key));
        let decoded = RsaHandshake::from_packet(&decoded_packet).unwrap();
        assert!(decoded.public_key().public_eq(&private_key));
    }
}
