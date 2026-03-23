use serde::{Deserialize, Serialize};

use crate::net::{bytehandler::{
    ByteReader, ByteWriter, Decode, Encode, error::PacketError
}, packet::event::chat_message::ChatMessagePacket};
pub mod chat_message;

/// Event subtypes
pub const EVENT_CHAT_MESSAGE: u8 = 1;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum EventPacket {
    ChatMessage(ChatMessagePacket),
}

impl Encode for EventPacket {
    fn encode(&self, w: &mut ByteWriter) {
        match self {
            EventPacket::ChatMessage(msg) => {
                w.write_u8(EVENT_CHAT_MESSAGE);
                msg.encode(w);
            }
        }
    }
}

impl Decode for EventPacket {
    fn decode(r: &mut ByteReader) -> Result<Self, PacketError> {
        match r.read_u8()? {
            EVENT_CHAT_MESSAGE => Ok(Self::ChatMessage(ChatMessagePacket::decode(r)?)),
            other => Err(PacketError::UnknownEventType(other)),
        }
    }
}
