use openssl::pkey::{PKey, Private, Public};

use super::packet::Packet;

pub struct Handshake {
    public_key: Vec<u8>,
}

impl Handshake {
    pub fn new(private_key: &PKey<Private>) -> Self {
        let public_key = private_key.public_key_to_pem().unwrap();
        Self { public_key }
    }

    pub fn from_public_key(public_key: Vec<u8>) -> Self {
        Self { public_key }
    }

    pub fn public_key(&self) -> PKey<Public> {
        PKey::public_key_from_pem(&self.public_key).unwrap()
    }

    pub fn into_packet(self, private_key: &PKey<Private>) -> Packet {
        Packet::from_data(self.public_key, private_key)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use openssl::pkey::PKey;
    use std::io::Cursor;

    const TEST_PRIVATE_KEY: &[u8] = b"-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIHQUR+jFSCq7hQZcvIS2/DUJWP5u0Y8Yq+aoSNl/eKyp\n-----END PRIVATE KEY-----\n";

    #[test]
    fn test_handshake_new() {
        let private_key = PKey::generate_ed25519().unwrap();
        let handshake = Handshake::new(&private_key);
        let public_key = handshake.public_key();
        assert!(public_key.public_eq(&private_key));
    }

    #[test]
    fn test_packet_conversion() {
        let private_key = PKey::private_key_from_pem(TEST_PRIVATE_KEY).unwrap();
        let handshake = Handshake::new(&private_key);
        let public_key = handshake.public_key();
        let packet = handshake.into_packet(&private_key);
        let mut data = Vec::new();
        packet.to_writer(&mut data).unwrap();
        let decoded_packet = Packet::from_reader(&mut Cursor::new(data)).unwrap();
        assert!(decoded_packet.verify(&public_key));
        let decoded: Handshake = decoded_packet.into();
    }
}
