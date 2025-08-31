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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, ErrorKind};

    #[test]
    fn test_write_and_read_buffer_roundtrip() {
        let data = b"hello world";
        let mut buf = Vec::new();
        write_buffer(&mut buf, data).unwrap();

        let mut cursor = Cursor::new(buf);
        let result = read_buffer(&mut cursor).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_read_buffer_empty() {
        let mut cursor = Cursor::new(Vec::new());
        let result = read_buffer(&mut cursor);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_read_buffer_insufficient_data() {
        // Write a length prefix for 10 bytes, but only provide 5 bytes
        let mut buf = Vec::new();
        buf.write_varint(10u32).unwrap();
        buf.extend_from_slice(&[1, 2, 3, 4, 5]);
        let mut cursor = Cursor::new(buf);
        let result = read_buffer(&mut cursor);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::UnexpectedEof);
    }
}
