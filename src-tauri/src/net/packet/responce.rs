use serde::{Deserialize, Serialize};

use crate::net::bytehandler::{
    error::PacketError,
    ByteReader, ByteWriter, Decode, Encode,
};

/// Response payload types
pub const RES_CAPABILITIES: u8 = 1;
pub const RES_OK: u8 = 2;
pub const RES_ERROR: u8 = 3;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ResponsePacket {
    Capabilities { caps: Vec<String> },
    Ok,
    Error { message: String },
}

impl Encode for ResponsePacket {
    fn encode(&self, w: &mut ByteWriter) {
        match self {
            ResponsePacket::Capabilities { caps } => {
                w.write_u8(RES_CAPABILITIES);
                w.write_u32(caps.len() as u32);
                for capability in caps {
                    w.write_string(capability);
                }
            }
            ResponsePacket::Ok => {
                w.write_u8(RES_OK);
            }
            ResponsePacket::Error { message } => {
                w.write_u8(RES_ERROR);
                w.write_string(message);
            }
        }
    }
}

impl Decode for ResponsePacket {
    fn decode(r: &mut ByteReader) -> Result<Self, PacketError> {
        match r.read_u8()? {
            RES_CAPABILITIES => {
                let count = r.read_u32()? as usize;
                let mut caps = Vec::with_capacity(count);
                for _ in 0..count {
                    caps.push(r.read_string()?);
                }
                Ok(Self::Capabilities { caps })
            }
            RES_OK => Ok(Self::Ok),
            RES_ERROR => Ok(Self::Error {
                message: r.read_string()?,
            }),
            other => Err(PacketError::UnknownResponseType(other)),
        }
    }
}
