use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    crypto::{LocalIdentity, peer_id_from_bytes}, models::ChatMessage, net::{Connection, packet::handshake::PeerIdentity}
};


pub struct Peer {
    pub peer_id: String,        // derived from their public_key
    pub identity: PeerIdentity, // from their handshake
    pub connection: Arc<Connection>,
    pub messages: Arc<Mutex<Vec<ChatMessage>>>,
    pub status: PeerStatus, // Online, Disconnected
}

pub enum PeerStatus {
    Connected,
    Disconnected,
}
impl Peer {
    pub fn new(identity: PeerIdentity, connection: Arc<Connection>, status: PeerStatus) -> Self {
        // TODO: TO Be removed unwrap
        let peer_id = peer_id_from_bytes(&identity.public_key_bytes).unwrap();
        let messages = Arc::new(Mutex::new(vec![]));

        Self {
            peer_id,
            identity,
            connection,
            messages,
            status,
        }
    }
    pub async fn get_messages(&self, skip: u32, limit: u32) -> Vec<ChatMessage> {
        let messages = self.messages.lock().await;

        messages
            .iter()
            .skip(skip as usize)
            .take(limit as usize)
            .cloned()
            .collect()
    }
}

pub fn local_peer_identity(l_id: &LocalIdentity) -> PeerIdentity {
    let display_name = {
        let username = whoami::username();
        if username.trim().is_empty() {
            let random_suffix = Uuid::new_v4().as_u128() % 1_000_000;
            format!("ghost_user_{random_suffix}")
        } else {
            username
        }
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    PeerIdentity {
        public_key_bytes: l_id.public_key_bytes(),
        display_name,
        client_version: env!("CARGO_PKG_VERSION").to_string(),
        capabilities: vec![
            "CLEAR_TEXT".to_string(),
            "NO_AUTH".to_string(),
            "SIMPLE_TEXT_CHAT".to_string(),
        ],
        timestamp,
    }
}
