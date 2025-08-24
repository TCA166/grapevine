use std::io::{self, Read, Write};

use bitcode::{deserialize, serialize};
use integer_encoding::{VarIntReader, VarIntWriter};
use openssl::{
    pkey::{PKey, Private, Public},
    sign::{Signer, Verifier},
};
use serde::{Deserialize, Serialize};

fn read_buffer<R: Read>(reader: &mut R) -> Result<Vec<u8>, io::Error> {
    let length = reader.read_varint::<u32>()?;
    let mut data = vec![0; length as usize];
    reader.read_exact(&mut data)?;

    Ok(data)
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
        let signature = signer.sign_oneshot_to_vec(&data).unwrap();

        Packet { data, signature }
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, io::Error> {
        let data = read_buffer(reader)?;
        let signature = read_buffer(reader)?;

        Ok(Packet { data, signature })
    }

    pub fn to_writer<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        write_buffer(writer, &self.data)?;
        write_buffer(writer, &self.signature)?;
        Ok(())
    }

    pub fn verify(&self, public_key: &PKey<Public>) -> bool {
        let mut verifier = Verifier::new_without_digest(public_key).unwrap();
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
