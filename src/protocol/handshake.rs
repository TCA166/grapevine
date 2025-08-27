use std::io::{self, Read, Write};

use bitcode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

use super::io::{read_buffer, write_buffer};

#[derive(Deserialize, Serialize)]
pub enum ProtocolPath {
    /// I don't know you, I would like to exchange RSA keys
    RsaExchange,
    /// I know you, you should know me
    AesExchange,
}

impl Default for ProtocolPath {
    fn default() -> Self {
        ProtocolPath::RsaExchange
    }
}

const PROTOCOL_V: u16 = 1;

#[derive(Deserialize, Serialize)]
pub struct Handshake {
    path: ProtocolPath,
    version: u16,
}

impl Handshake {
    pub fn new(path: ProtocolPath) -> Self {
        Self {
            path,
            version: PROTOCOL_V,
        }
    }

    pub fn version_ok(&self) -> bool {
        self.version == PROTOCOL_V
    }

    pub fn next(self) -> ProtocolPath {
        self.path
    }

    pub fn to_writer<W: Write>(&self, stream: &mut W) -> Result<(), io::Error> {
        write_buffer(stream, &serialize(self).unwrap())
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, io::Error> {
        Ok(deserialize(&read_buffer(reader)?).unwrap())
    }
}

impl Default for Handshake {
    fn default() -> Self {
        Self::new(ProtocolPath::default())
    }
}
