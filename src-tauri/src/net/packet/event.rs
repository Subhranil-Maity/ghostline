use serde::{Deserialize, Serialize};

use crate::net::bytehandler::{
    error::PacketError,
    ByteReader, ByteWriter, Decode, Encode,
};

/// Event subtypes
pub const EVENT_CHAT_MESSAGE: u8 = 1;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum EventPacket {
    ChatMessage(String),
}

impl Encode for EventPacket {
    fn encode(&self, w: &mut ByteWriter) {
        match self {
            EventPacket::ChatMessage(msg) => {
                w.write_u8(EVENT_CHAT_MESSAGE);
                w.write_string(msg);
            }
        }
    }
}

impl Decode for EventPacket {
    fn decode(r: &mut ByteReader) -> Result<Self, PacketError> {
        match r.read_u8()? {
            EVENT_CHAT_MESSAGE => Ok(Self::ChatMessage(r.read_string()?)),
            other => Err(PacketError::UnknownEventType(other)),
        }
    }
}
