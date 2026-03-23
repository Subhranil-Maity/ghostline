use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    crypto::LocalIdentity,
    net::packet::handshake::PeerIdentity,
    peer::{local_peer_identity, Peer},
};

pub struct AppState {
    pub server: RwLock<Option<Arc<crate::net::Server>>>,
    pub local_peer: Arc<PeerIdentity>,
    pub peers: Arc<Mutex<HashMap<String, Arc<Peer>>>>,
    pub local_identity: Arc<LocalIdentity>,
}
impl AppState {
    pub fn new(local_identity: LocalIdentity) -> Self {
        Self {
            server: RwLock::new(None),
            local_peer: Arc::new(local_peer_identity(&local_identity)),
            peers: Arc::new(Mutex::new(HashMap::new())),
            local_identity: Arc::new(local_identity),
        }
    }
}
