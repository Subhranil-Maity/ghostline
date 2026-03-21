pub mod error;

use crate::net::bytehandler::error::PacketError;

pub struct ByteWriter {
    buf: Vec<u8>,
}

pub trait Encode {
    fn encode(&self, w: &mut ByteWriter);
}

pub trait Decode: Sized {
    fn decode(r: &mut ByteReader) -> Result<Self, PacketError>;
}

impl ByteWriter {
    pub fn new() -> Self {
        Self { buf: vec![] }
    }

    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn write_u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn write_bytes(&mut self, v: &[u8]) {
        self.buf.extend_from_slice(v);
    }

    pub fn write_string(&mut self, s: &str) {
        self.write_u32(s.len() as u32);
        self.write_bytes(s.as_bytes());
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }
}

pub struct ByteReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn take(&mut self, len: usize, reading: &'static str) -> Result<&'a [u8], PacketError> {
        let remaining = self.buf.len().saturating_sub(self.pos);
        if remaining < len {
            return Err(PacketError::UnexpectedEof {
                reading,
                needed: len,
                remaining,
            });
        }

        let start = self.pos;
        let end = start + len;
        self.pos = end;
        Ok(&self.buf[start..end])
    }

    pub fn read_u8(&mut self) -> Result<u8, PacketError> {
        Ok(self.take(1, "u8")?[0])
    }

    pub fn read_u32(&mut self) -> Result<u32, PacketError> {
        let bytes = self.take(4, "u32")?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_u64(&mut self) -> Result<u64, PacketError> {
        let bytes = self.take(8, "u64")?;
        Ok(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn read_string(&mut self) -> Result<String, PacketError> {
        let len = self.read_u32()? as usize;
        let bytes = self.take(len, "string bytes")?;
        String::from_utf8(bytes.to_vec()).map_err(|source| PacketError::InvalidUtf8 {
            field: "string",
            source,
        })
    }

    pub fn remaining(&self) -> &[u8] {
        &self.buf[self.pos..]
    }
}
