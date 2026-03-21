use std::{
    time::{SystemTime, UNIX_EPOCH},
};

use uuid::Uuid;

use crate::net::packet::handshake::PeerIdentity;

pub fn local_peer_identity() -> PeerIdentity {
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
        peer_id: Uuid::new_v4().to_string(),
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
