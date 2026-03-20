mod net;
mod state;
use std::sync::Arc;

use state::AppState;
use tauri::State;

use crate::net::utils::send_simple_text_packet;
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
async fn connect_to_host(state: State<'_, AppState>, addr: String) -> Result<bool, String> {
    let conn = net::Client::new(addr.clone());
    let connection = conn.connect().await.map_err(|e| e.to_string())?;
    let connection = Arc::new(connection);
    println!("Connected To: {}", addr);
    {
        let mut connections = state.connections.lock().unwrap();
        connections.insert(addr, connection);
    }
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

    conn.message_history
        .lock()
        .await
        .push(("You".to_string(), msg));

    Ok(())
}
// TODO: FAst use broadcasing form connection class it sself to manage state changes

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let state = AppState::new();
    let server = Arc::new(net::Server::new("0.0.0.0:8000"));
    {
        let mut server_lock = state.server.write().unwrap();
        *server_lock = Some(server.clone());
    }
    let conns = state.connections.clone();
    tokio::spawn(async move {
        server
            .start(move |connection, addr| {
                conns.lock().unwrap().insert(addr, Arc::new(connection));
            })
            .await
            .unwrap();
    });
    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_opener::init())
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
