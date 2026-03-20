use std::{collections::HashMap, sync::{Arc, Mutex, RwLock}};

use crate::net::Connection;

pub struct AppState {
    pub server: RwLock<Option<Arc<crate::net::Server>>>,
    // TODO: Instade of raw connection use a Indentity clas that will hold teh actual msg history
    // and other things sych as the actual broadcaster
    pub connections: Arc<Mutex<HashMap<String, Arc<Connection>>>>
    // pub client: Option<crate::client::Client>,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            server: RwLock::new(None),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
}
