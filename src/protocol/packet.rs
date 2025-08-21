use std::io::{Read, Write};

use integer_encoding::{VarIntReader, VarIntWriter};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private, Public},
    sign::{Signer, Verifier},
};

use super::{handshake::Handshake, message::Message};

fn read_buffer<R: Read>(reader: &mut R) -> Option<Vec<u8>> {
    let length = reader.read_varint::<u32>().ok()?;
    let mut data = vec![0; length as usize];
    reader.read_exact(&mut data).ok()?;

    Some(data)
}

fn write_buffer<W: Write>(writer: &mut W, data: &[u8]) -> std::io::Result<()> {
    writer.write_varint(data.len() as u32)?;
    writer.write_all(data)
}

pub struct Packet {
    data: Vec<u8>,
    signature: Vec<u8>,
}

impl Packet {
    pub fn from_data(data: Vec<u8>, private_key: &PKey<Private>) -> Self {
        let mut signer = Signer::new_without_digest(private_key).unwrap();
        signer.update(&data).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        Packet { data, signature }
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        let data = read_buffer(reader)?;
        let signature = read_buffer(reader)?;

        Some(Packet { data, signature })
    }

    pub fn to_writer<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write_buffer(writer, &self.data)?;
        write_buffer(writer, &self.signature)?;
        Ok(())
    }

    pub fn verify(&self, public_key: &PKey<Public>) -> bool {
        let mut verifier = Verifier::new(MessageDigest::sha256(), public_key).unwrap();
        verifier.update(&self.data).unwrap();
        verifier.verify(&self.signature).unwrap_or(false)
    }
}

impl Into<Message> for Packet {
    fn into(self) -> Message {
        let mut parts = self.data.splitn(2, |&b| b == 0);
        let author =
            String::from_utf8(parts.next().unwrap_or_default().to_vec()).unwrap_or_default();
        let message =
            String::from_utf8(parts.next().unwrap_or_default().to_vec()).unwrap_or_default();
        Message::new(author, message)
    }
}

impl Into<Handshake> for Packet {
    fn into(self) -> Handshake {
        Handshake::from_public_key(self.data.to_vec())
    }
}
