use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    crypto::LocalIdentity,
    net::{packet::handshake::PeerIdentity, Connection},
    peer::{local_peer_identity, Peer},
};

pub struct AppState {
    pub server: RwLock<Option<Arc<crate::net::Server>>>,
    pub local_peer: Arc<PeerIdentity>,
    // TODO: Instade of raw connection use a Indentity clas that will hold teh actual msg history
    // and other things sych as the actual broadcaster
    pub peers: Arc<Mutex<HashMap<String, Arc<Peer>>>>,
    pub local_identity: Arc<LocalIdentity>,
    // pub client: Option<crate::client::Client>,
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
