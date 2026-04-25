mod chat_message;
pub use chat_message::ChatMessage;
pub use chat_message::MessageSender;
use serde::Deserialize;
use serde::Serialize;

use crate::peer::PeerStatus;



#[derive(Clone, Serialize, Deserialize)]
pub enum Message{
    SimpleTextMessage(ChatMessage),
    PeerStatusUpdated(PeerStatus)
}
