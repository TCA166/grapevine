use std::{
    error,
    io::{self, Read, Write},
};

use bitcode::{self, deserialize, serialize};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private, Public},
    sign::{Signer, Verifier},
    symm::{Cipher, decrypt, encrypt},
};
use serde::{Deserialize, Serialize};

use super::{
    AesIv, AesKey,
    io::{read_buffer, write_buffer},
    new_aes_iv,
};

/// Structured primitive data carrier
/// Each packet is signed, and optionally encrypted. If encrypted it also
/// carries the `iv`.
pub struct Packet {
    data: Vec<u8>,
    signature: Vec<u8>,
    iv: Option<AesIv>,
}

impl Packet {
    /// Creates a new packet based on the data, and signs it with the provided private key.
    fn from_data(data: Vec<u8>, private_key: &PKey<Private>) -> Self {
        let mut signer = Signer::new(MessageDigest::sha256(), private_key).unwrap();
        let signature = signer.sign_oneshot_to_vec(&data).unwrap();

        Packet {
            data,
            signature,
            iv: None,
        }
    }

    /// Reads a packet from a reader.
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, io::Error> {
        let data = read_buffer(reader)?;
        let signature = read_buffer(reader)?;
        let potential_iv = read_buffer(reader)?;
        let iv = if potential_iv.is_empty() {
            None
        } else {
            Some(potential_iv.try_into().unwrap())
        };

        Ok(Packet {
            data,
            signature,
            iv,
        })
    }

    /// Writes a packet to a writer.
    pub fn to_writer<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        write_buffer(writer, &self.data)?;
        write_buffer(writer, &self.signature)?;
        if let Some(iv) = &self.iv {
            write_buffer(writer, iv)?;
        } else {
            write_buffer(writer, &[])?;
        }
        Ok(())
    }

    /// Encrypts the packet
    pub fn encrypt(&mut self, other_aes_key: &AesKey) -> Result<(), io::Error> {
        let cipher = Cipher::aes_256_cbc();
        let iv = new_aes_iv()?;

        self.data = encrypt(cipher, other_aes_key, Some(&iv), &self.data)?;
        self.signature = encrypt(cipher, other_aes_key, Some(&iv), &self.signature)?;
        self.iv = Some(iv);
        Ok(())
    }

    /// Decrypts the packet
    pub fn decrypt(&mut self, our_aes_key: &AesKey) -> Result<(), io::Error> {
        let cipher = Cipher::aes_256_cbc();
        let iv: Option<&[u8]> = if let Some(iv) = &self.iv {
            Some(iv)
        } else {
            None
        };

        self.data = decrypt(cipher, our_aes_key, iv, &self.data)?;
        self.signature = decrypt(cipher, our_aes_key, iv, &self.signature)?;
        Ok(())
    }

    /// Verifies the signature of the packet
    pub fn verify(&self, public_key: &PKey<Public>) -> bool {
        let mut verifier = Verifier::new(MessageDigest::sha256(), public_key).unwrap();
        verifier
            .verify_oneshot(&self.signature, &self.data)
            .unwrap_or(false)
    }
}

pub trait IntoPacket<E: error::Error> {
    fn into_packet(&self, private_key: &PKey<Private>) -> Result<Packet, E>;
}

impl<T: Serialize> IntoPacket<bitcode::Error> for T {
    fn into_packet(&self, private_key: &PKey<Private>) -> Result<Packet, bitcode::Error> {
        Ok(Packet::from_data(serialize(self)?, private_key))
    }
}

pub trait FromPacket<'de, E: error::Error>: Sized {
    fn from_packet(packet: &'de Packet) -> Result<Self, E>;
}

impl<'de, T: Deserialize<'de>> FromPacket<'de, bitcode::Error> for T {
    fn from_packet(packet: &'de Packet) -> Result<T, bitcode::Error> {
        deserialize(&packet.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openssl::pkey::PKey;
    use openssl::rsa::Rsa;
    use std::io::Cursor;

    #[test]
    fn test_packet_write_and_read_roundtrip() {
        let rsa = Rsa::generate(2048).unwrap();
        let private = PKey::from_rsa(rsa).unwrap();
        let data = b"test data".to_vec();
        let packet = Packet::from_data(data.clone(), &private);

        let mut buf = Vec::new();
        packet.to_writer(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let read_packet = Packet::from_reader(&mut cursor).unwrap();

        assert_eq!(read_packet.data, data);
        assert_eq!(read_packet.signature, packet.signature);
        assert_eq!(read_packet.iv, packet.iv);
    }

    #[test]
    fn test_packet_signature_verification() {
        let rsa = Rsa::generate(2048).unwrap();
        let private = PKey::from_rsa(rsa).unwrap();
        let public = PKey::public_key_from_pem(&private.public_key_to_pem().unwrap()).unwrap();

        let data = b"verify me".to_vec();
        let packet = Packet::from_data(data, &private);

        assert!(packet.verify(&public));
    }

    #[test]
    fn test_packet_signature_verification_fails_on_tamper() {
        let rsa = Rsa::generate(2048).unwrap();
        let private = PKey::from_rsa(rsa).unwrap();
        let public = PKey::public_key_from_pem(&private.public_key_to_pem().unwrap()).unwrap();

        let data = b"verify me".to_vec();
        let mut packet = Packet::from_data(data, &private);

        // Tamper with the data
        packet.data[0] ^= 0xFF;
        assert!(!packet.verify(&public));
    }

    #[test]
    fn test_packet_encrypt_decrypt_roundtrip() {
        let rsa = Rsa::generate(2048).unwrap();
        let private = PKey::from_rsa(rsa).unwrap();
        let data = b"secret data".to_vec();
        let mut packet = Packet::from_data(data.clone(), &private);

        // Generate a random AES key (32 bytes for AES-256)
        let aes_key: AesKey = [42u8; 32];

        packet.encrypt(&aes_key).unwrap();
        assert!(packet.iv.is_some());
        // Data and signature should be encrypted (not equal to original)
        assert_ne!(packet.data, data);

        packet.decrypt(&aes_key).unwrap();
        assert_eq!(packet.data, data);
    }

    #[test]
    fn test_into_packet_and_from_packet_traits() {
        let rsa = Rsa::generate(2048).unwrap();
        let private = PKey::from_rsa(rsa).unwrap();

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Dummy {
            a: u32,
            b: String,
        }

        let dummy = Dummy {
            a: 42,
            b: "hello".to_string(),
        };
        let packet = dummy.into_packet(&private).unwrap();
        let recovered: Dummy = <Dummy as FromPacket<bitcode::Error>>::from_packet(&packet).unwrap();
        assert_eq!(dummy, recovered);
    }
}
