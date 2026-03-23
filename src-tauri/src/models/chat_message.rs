use serde::{Deserialize, Serialize};

use crate::net::packet::event::chat_message::ChatMessagePacket;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub uuid: String,                  
    pub content: String,
    pub timestamp: u64,         
    pub sender: MessageSender,
    // pub status: MessageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSender {
    Me,
    Remote,
}
impl ChatMessage {
    pub fn from_packet(pkt: ChatMessagePacket, sender: MessageSender) -> ChatMessage{
         ChatMessage {
             uuid: pkt.uuid,
             content: pkt.content,
             timestamp: pkt.timestamp,
             sender: sender, // receiving a packet means it came from remote
         }
        
    }
}
// impl From<ChatMessagePacket> for ChatMessage {
//     fn from(pkt: ChatMessagePacket) -> Self {
//         ChatMessage {
//             uuid: pkt.uuid,
//             content: pkt.content,
//             timestamp: pkt.timestamp,
//             sender: MessageSender::Remote, // receiving a packet means it came from remote
//         }
//     }
// }
