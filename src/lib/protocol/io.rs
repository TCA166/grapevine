use std::io::{self, Read, Write};

use integer_encoding::{VarIntReader, VarIntWriter};

/// Reads a buffer from a stream
pub fn read_buffer<R: Read>(reader: &mut R) -> Result<Vec<u8>, io::Error> {
    let length = reader.read_varint::<u32>()?;
    let mut data = vec![0; length as usize];
    reader.read_exact(&mut data)?;

    Ok(data)
}

/// Writes a buffer into a stream
pub fn write_buffer<W: Write>(writer: &mut W, data: &[u8]) -> std::io::Result<()> {
    writer.write_varint(data.len() as u32)?;
    writer.write_all(data)
}
