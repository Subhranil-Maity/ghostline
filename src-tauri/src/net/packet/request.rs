use serde::{Deserialize, Serialize};

use crate::net::bytehandler::{
    error::PacketError,
    ByteReader, ByteWriter, Decode, Encode,
};

/// Request payload types
pub const REQ_GET_CAPABILITIES: u8 = 1;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum RequestPacket {
    GetCapabilities,
}

impl Encode for RequestPacket {
    fn encode(&self, w: &mut ByteWriter) {
        match self {
            RequestPacket::GetCapabilities => w.write_u8(REQ_GET_CAPABILITIES),
        }
    }
}

impl Decode for RequestPacket {
    fn decode(r: &mut ByteReader) -> Result<Self, PacketError> {
        match r.read_u8()? {
            REQ_GET_CAPABILITIES => Ok(Self::GetCapabilities),
            other => Err(PacketError::UnknownRequestType(other)),
        }
    }
}
