use std::{collections::HashMap, sync::{Arc, Mutex, RwLock}};

use crate::{net::{packet::handshake::PeerIdentity, Connection}, peer::local_peer_identity};

pub struct AppState {
    pub server: RwLock<Option<Arc<crate::net::Server>>>,
    pub local_peer: Arc<PeerIdentity>,
    // TODO: Instade of raw connection use a Indentity clas that will hold teh actual msg history
    // and other things sych as the actual broadcaster
    pub connections: Arc<Mutex<HashMap<String, Arc<Connection>>>>
    // pub client: Option<crate::client::Client>,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            server: RwLock::new(None),
            local_peer: Arc::new(local_peer_identity()),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
}
