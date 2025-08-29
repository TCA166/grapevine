use std::io::{self, Read, Write};

use bitcode::{deserialize, serialize};
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
    new_aes_iv
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

pub trait IntoPacket {
    fn into_packet(&self, private_key: &PKey<Private>) -> Packet;
}

impl<T: Serialize> IntoPacket for T {
    fn into_packet(&self, private_key: &PKey<Private>) -> Packet {
        let data = serialize(self).unwrap();
        Packet::from_data(data, private_key)
    }
}

pub trait FromPacket<'de> {
    fn from_packet(packet: &'de Packet) -> Self;
}

impl<'de, T: Deserialize<'de>> FromPacket<'de> for T {
    fn from_packet(packet: &'de Packet) -> T {
        deserialize(&packet.data).unwrap()
    }
}
