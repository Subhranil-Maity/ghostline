mod net;
mod state;
mod peer;
use std::sync::Arc;

use serde::Serialize;
use state::AppState;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;

use crate::net::{
    packet::handshake::PeerIdentity,
    utils::send_simple_text_packet,
    Connection, ConnectionEvent,
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
    spawn_connection_event_handler(
        connection,
        event_rx,
        addr,
        state.connections.clone(),
        app,
    );
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
        let connection = {
            let c = state.connections.lock().unwrap();
            c.get(&id).cloned()
        };

        if let Some(conn) = connection {
            let c = conn.get_messages(skip, limit).await;
            chats.extend(c);
        }
    }

    Ok(chats)
}
#[tauri::command(async)]
async fn get_my_connections(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let conns = state.connections.lock().unwrap();
    let addresses = conns.keys().cloned().collect();
    Ok(addresses)
}

#[tauri::command(async)]
async fn send_simple_text(
    state: State<'_, AppState>,
    conn_id: String,
    msg: String,
) -> Result<(), String> {
    let conn = {
        let c = state.connections.lock().unwrap();
        c.get(&conn_id)
            .cloned()
            .ok_or_else(|| "Connection not found".to_string())?
    };

    // let conn = connections

    // send_simple_text_packet.(msg).await.map_err(|e| e.to_string())?;
    //TODO: NEED TO SIMPLIFY &*
    send_simple_text_packet(&*conn, msg.clone())
        .await
        .map_err(|e| e.to_string())?;

    conn.event_sender()
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
    connections: Arc<std::sync::Mutex<std::collections::HashMap<String, Arc<Connection>>>>,
    app_handle: tauri::AppHandle,
) {
    tokio::spawn(async move {
        let mut connection_id = pending_addr;
        while let Some(event) = event_rx.recv().await {
            match event {
                ConnectionEvent::PeerIdentified { peer } => {
                    connection_id = register_connection(
                        &connections,
                        connection.clone(),
                        &peer,
                    );
                    let _ = app_handle.emit(
                        CONNECTION_CREATED_EVENT,
                        FrontendConnectionEvent {
                            connection_id: connection_id.clone(),
                        },
                    );
                }
                ConnectionEvent::MessageReceived { from, message } => {
                    connection
                        .message_history
                        .lock()
                        .await
                        .push((from.clone(), message.clone()));
                    let _ = app_handle.emit(
                        MESSAGE_RECEIVED_EVENT,
                        FrontendMessageEvent {
                            connection_id: connection_id.clone(),
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
    connections: &Arc<std::sync::Mutex<std::collections::HashMap<String, Arc<Connection>>>>,
    connection: Arc<Connection>,
    peer: &PeerIdentity,
) -> String {
    let mut guard = connections.lock().unwrap();
    guard.insert(peer.peer_id.clone(), connection);
    peer.peer_id.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let state = AppState::new();
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
            let conns = state.connections.clone();
            let app_handle = app.handle().clone();

            tokio::spawn(async move {
                server
                    .start(move |connection, event_rx, addr| {
                        let connection = Arc::new(connection);
                        spawn_connection_event_handler(
                            connection.clone(),
                            event_rx,
                            addr.clone(),
                            conns.clone(),
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
