use serde::{Deserialize, Serialize};

use crate::{models::ChatMessage, net::bytehandler::{Decode, Encode}};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessagePacket {
    pub uuid: String,
    pub content: String,
    pub timestamp: u64,
}

impl Encode for ChatMessagePacket {
    fn encode(&self, w: &mut crate::net::bytehandler::ByteWriter) {
        w.write_string(&self.uuid);
        w.write_string(&self.content);
        w.write_u64(self.timestamp);
    }
}

impl Decode for ChatMessagePacket {
    fn decode(
        r: &mut crate::net::bytehandler::ByteReader,
    ) -> Result<Self, crate::net::bytehandler::error::PacketError> {
        let uuid = r.read_string()?;
        let content = r.read_string()?;
        let timestamp = r.read_u64()?;
        Ok(Self {
            uuid,
            content,
            timestamp,
        })
    }
}
impl From<ChatMessage> for ChatMessagePacket {
    fn from(msg: ChatMessage) -> Self {
        ChatMessagePacket {
            uuid: msg.uuid,
            content: msg.content,
            timestamp: msg.timestamp,
            // owner dropped, not sent on wire
        }
    }
}
