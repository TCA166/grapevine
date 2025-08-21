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
