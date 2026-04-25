use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::{
    crypto::{LocalIdentity, peer_id_from_bytes}, models::{ChatMessage, Message}, net::{Connection, packet::handshake::PeerIdentity}
};


pub struct Peer {
    pub peer_id: String,        // derived from their public_key
    pub identity: RwLock<PeerIdentity>, // from their handshake
    pub connection: RwLock<Arc<Connection>>,
    pub messages: Arc<Mutex<Vec<Message>>>,
    pub status: RwLock<PeerStatus>, // Online, Disconnected
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
            identity: RwLock::new(identity),
            connection: RwLock::new(connection),
            messages,
            status: RwLock::new(status),
        }
    }

    pub async fn connection(&self) -> Arc<Connection> {
        self.connection.read().await.clone()
    }

    pub async fn replace_connection(&self, identity: PeerIdentity, connection: Arc<Connection>) {
        *self.identity.write().await = identity;
        *self.connection.write().await = connection;
        *self.status.write().await = PeerStatus::Connected;
    }

    pub async fn set_status(&self, status: PeerStatus) {
        *self.status.write().await = status;
    }

    pub async fn status(&self) -> PeerStatus {
        *self.status.read().await
    }

    pub async fn get_messages(&self, skip: u32, limit: u32) -> Vec<Message> {
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
            // "NO_AUTH".to_string(),
            "SIMPLE_TEXT_CHAT".to_string(),
        ],
        timestamp,
    }
}
