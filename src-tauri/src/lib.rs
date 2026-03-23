mod crypto;
mod net;
mod peer;
mod state;
use std::{path::PathBuf, sync::Arc};

use serde::Serialize;
use state::AppState;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;

use crate::{
    crypto::{peer_id_from_bytes, LocalIdentity},
    net::{
        packet::handshake::PeerIdentity, utils::send_simple_text_packet, Connection,
        ConnectionEvent,
    },
    peer::Peer,
};

const MESSAGE_RECEIVED_EVENT: &str = "ghostline://message-received";
const CONNECTION_CREATED_EVENT: &str = "ghostline://connection-created";

#[derive(Clone, Serialize)]
struct FrontendMessageEvent {
    connection_id: String,
    from: String,
    message: String,
}

#[derive(Clone, Serialize)]
struct FrontendConnectionEvent {
    connection_id: String,
}
// type TauriAppState = tauri::State<'_, Mutex<AppState>>;
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
// TODO: TRACEBACK OF IF a message is realy sent as connection can drop
#[tauri::command]
fn get_server_address(state: State<'_, AppState>) -> String {
    let server_guard = state.server.read().unwrap();
    if let Some(server) = &*server_guard {
        server.get_address()
    } else {
        "Server not running".to_string()
    }
}

#[tauri::command(async)]
async fn connect_to_host(
    app: AppHandle,
    state: State<'_, AppState>,
    addr: String,
) -> Result<bool, String> {
    let conn = net::Client::new(addr.clone(), state.local_peer.as_ref().clone());
    let (connection, event_rx) = conn.connect().await.map_err(|e| e.to_string())?;
    let connection = Arc::new(connection);
    spawn_connection_event_handler(connection, event_rx, addr, state.peers.clone(), app);
    Ok(true)
}

#[tauri::command(async)]
async fn get_connection_messages(
    state: State<'_, AppState>,
    id: String,
    limit: u32,
    skip: u32,
) -> Result<Vec<(String, String)>, String> {
    let mut chats: Vec<(String, String)> = vec![];

    {
        let peer = {
            let p = state.peers.lock().unwrap();
            p.get(&id).cloned()
        };

        if let Some(c) = peer {
            let p = c.get_messages(skip, limit).await;
            chats.extend(p);
        }
    }

    Ok(chats)
}
#[tauri::command(async)]
async fn get_my_connections(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let conns = state.peers.lock().unwrap();
    let ids = conns.keys().cloned().collect();
    Ok(ids)
}

#[tauri::command(async)]
async fn send_simple_text(
    state: State<'_, AppState>,
    conn_id: String,
    msg: String,
) -> Result<(), String> {
    let peer = {
        let c = state.peers.lock().unwrap();
        c.get(&conn_id)
            .cloned()
            .ok_or_else(|| "Connection not found".to_string())?
    };

    // let conn = connections

    // send_simple_text_packet.(msg).await.map_err(|e| e.to_string())?;
    //TODO: NEED TO SIMPLIFY &*
    send_simple_text_packet(&peer.connection, msg.clone())
        .await
        .map_err(|e| e.to_string())?;

    peer.connection
        .event_sender()
        .send(ConnectionEvent::MessageReceived {
            from: "You".to_string(),
            message: msg,
        })
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn spawn_connection_event_handler(
    connection: Arc<Connection>,
    mut event_rx: mpsc::Receiver<ConnectionEvent>,
    pending_addr: String,
    peers: Arc<std::sync::Mutex<std::collections::HashMap<String, Arc<Peer>>>>,
    app_handle: tauri::AppHandle,
) {
    tokio::spawn(async move {
        let mut peer_id = pending_addr;
        while let Some(event) = event_rx.recv().await {
            match event {
                ConnectionEvent::PeerIdentified { peer } => {
                    peer_id = register_connection(&peers, connection.clone(), &peer);
                    let _ = app_handle.emit(
                        CONNECTION_CREATED_EVENT,
                        FrontendConnectionEvent {
                            connection_id: peer_id.clone(),
                        },
                    );
                }
                ConnectionEvent::MessageReceived { from, message } => {
                    let peer = {
                        let p = peers.lock().unwrap();
                        p.get(&peer_id).cloned()
                    };
                    // drop(gaurd);
                    if let Some(peer) = peer {
                        peer
                            .messages
                            .lock()
                            .await
                            .push((from.clone(), message.clone()));
                    }

                    let _ = app_handle.emit(
                        MESSAGE_RECEIVED_EVENT,
                        FrontendMessageEvent {
                            connection_id: peer_id.clone(),
                            from,
                            message,
                        },
                    );
                }
                ConnectionEvent::CapabilitiesUpdated { caps } => {
                    println!("Connection capabilities updated: {:?}", caps);
                }
            }
        }
    });
}

fn register_connection(
    peers: &Arc<std::sync::Mutex<std::collections::HashMap<String, Arc<Peer>>>>,
    connection: Arc<Connection>,
    peer_identity: &PeerIdentity,
) -> String {
    let peer = Peer::new(
        peer_identity.clone(),
        connection,
        peer::PeerStatus::Connected,
    );
    let id = peer.peer_id.clone();
    let mut guard = peers.lock().unwrap();
    guard.insert(id.clone(), Arc::new(peer));
    id
}
fn keypair_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?; // returns None early if not found
    Some(home.join(".ghostline").join("identity.key"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    //TODO: HANDLE Gracefully
    let local_identity =
        LocalIdentity::load_or_generate(&keypair_path().expect("Could not get to config folder"))
            .unwrap();

    let state = AppState::new(local_identity);
    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state: State<'_, AppState> = app.state();
            let server = Arc::new(net::Server::new(
                "0.0.0.0:8000",
                state.local_peer.as_ref().clone(),
            ));
            {
                let mut server_lock = state.server.write().unwrap();
                *server_lock = Some(server.clone());
            }
            let peers = state.peers.clone();
            let app_handle = app.handle().clone();

            tokio::spawn(async move {
                server
                    .start(move |connection, event_rx, addr| {
                        let connection = Arc::new(connection);
                        spawn_connection_event_handler(
                            connection.clone(),
                            event_rx,
                            addr.clone(),
                            peers.clone(),
                            app_handle.clone(),
                        );
                    })
                    .await
                    .unwrap();
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_server_address,
            get_my_connections,
            get_connection_messages,
            send_simple_text,
            connect_to_host
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
