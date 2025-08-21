use std::io::{Read, Write};

use integer_encoding::{VarIntReader, VarIntWriter};

use super::message::Message;

pub struct Packet {
    length: u32,
    data: Vec<u8>,
}

impl Packet {
    pub fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        if let Ok(length) = reader.read_varint() {
            let mut data = vec![0; length as usize];
            reader.read_exact(&mut data).unwrap();
            Some(Packet { length, data })
        } else {
            None
        }
    }

    pub fn to_writer<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_varint(self.length)?;
        writer.write_all(&self.data)?;
        Ok(())
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
