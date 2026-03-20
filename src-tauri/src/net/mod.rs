mod client;
mod packet;
mod pending_requests;
mod server;
pub mod utils;

pub use client::Client;
pub use server::Server;

use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::{mpsc, oneshot, Mutex},
};

use crate::net::packet::{EventPacket, Packet, RequestPacket, ResponsePacket, decode};
type PendingRequest = Arc<Mutex<HashMap<u64, oneshot::Sender<ResponsePacket>>>>;
type MessageHistory = Arc<Mutex<Vec<(String, String)>>>;

#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    MessageReceived { from: String, message: String },
    CapabilitiesUpdated { caps: Vec<String> },
}

pub struct Connection {
    writer: Arc<Mutex<OwnedWriteHalf>>,
    pending_requests: PendingRequest,
    next_request_id: AtomicU64,
    connection_capabilities: Arc<Mutex<Vec<String>>>,
    pub message_history: MessageHistory,
    event_tx: mpsc::Sender<ConnectionEvent>,
}
const SIMPLE_TEXT_CHAT: &str = "SIMPLE_TEXT_CHAT";
impl Connection {
    pub fn new(mut reader: OwnedReadHalf, writer: OwnedWriteHalf) -> (Self, mpsc::Receiver<ConnectionEvent>) {
        let writer = Arc::new(Mutex::new(writer));
        let pending_requests: PendingRequest = Arc::new(Mutex::new(HashMap::new()));
        let (event_tx, event_rx) = mpsc::channel(32);

        let pending_clone = pending_requests.clone();
        let write_cloen = writer.clone();
        let event_tx_clone = event_tx.clone();
        let conn_cap = Arc::new(Mutex::new(vec![]));
        let conn_cap_clone = conn_cap.clone();
        let message_history: MessageHistory = Arc::new(Mutex::new(vec![]));
        let obj = Self {
            writer,
            pending_requests,
            next_request_id: AtomicU64::new(1),
            connection_capabilities: conn_cap,
            message_history,
            event_tx,
        };

        tokio::spawn(async move {
            let capabilities = vec!["CLEAR_TEXT".to_string(), "NO_AUTH".to_string(), SIMPLE_TEXT_CHAT.into()];
            let cap = packet::ResponsePacket::Capabilities { caps: capabilities };
            // TODO: request_id 0 means ist first packet with all initial data leter will be handled with
            // special case
            let packet = Packet::Response {
                request_id: 0,
                payload: cap,
            };

            let bytes = packet.encode();
            let mut writer = write_cloen.lock().await;
            writer.write_all(&bytes).await.unwrap();
            drop(writer);
            let mut buf = [0u8; 4096];

            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        // Deserialize incoming packet
                        if let Some(packet) = decode(&buf[..n]) {
                            println!("recived packet: {:#?}", packet);
                            match packet {
                                Packet::Response {
                                    request_id,
                                    payload,
                                } => {
                                    if request_id != 0 {
                                        let mut pending = pending_clone.lock().await;

                                        if let Some(tx) = pending.remove(&request_id) {
                                            let _ = tx.send(payload);
                                            return; // payload moved, so exit early
                                        }
                                    }

                                    // TODO: handle special case of first message in future will be
                                    // Enhanaced
                                    match &payload {
                                        ResponsePacket::Capabilities { caps } => {
                                            let mut conn_cap = conn_cap_clone.lock().await;
                                            *conn_cap = caps.clone();
                                            let _ = event_tx_clone
                                                .send(ConnectionEvent::CapabilitiesUpdated {
                                                    caps: caps.clone(),
                                                })
                                                .await;
                                        }
                                        _ => {}
                                    }
                                }

                                Packet::Event(event) => {
                                    // handle events (chat, ping, etc)
                                    println!("event: {:?}", event);
                                    match event {
                                        EventPacket::ChatMessage(msg) => {
                                            let _ = event_tx_clone
                                                .send(ConnectionEvent::MessageReceived {
                                                    from: "Other".to_string(),
                                                    message: msg,
                                                })
                                                .await;
                                        }
                                    }
                                }

                                Packet::Request {
                                    request_id,
                                    payload,
                                } => {
                                    // handle incoming request
                                    println!("request: {:?}", payload);

                                    // Example: respond with OK
                                    let _response = Packet::Response {
                                        request_id,
                                        payload: ResponsePacket::Ok,
                                    };

                                    todo!()
                                    // NOTE: writer is not available here; normally you'd route
                                    // requests to a handler that has access to the connection
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        (obj, event_rx)
    }
    pub async fn get_messages(&self, skip: u32, limit: u32) -> Vec<(String, String)> {
        let messages = self.message_history.lock().await;

        messages
            .iter()
            .skip(skip as usize)
            .take(limit as usize)
            .cloned()
            .collect()
    }



    pub async fn send_packet(&self, packet: Packet) -> io::Result<()> {
        let bytes = packet.encode();
        // .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "serialize error"))?;
        let mut writer = self.writer.lock().await;
        writer.write_all(&bytes).await
    }

    pub fn event_sender(&self) -> mpsc::Sender<ConnectionEvent> {
        self.event_tx.clone()
    }

    fn next_request_id(&self) -> u64 {
        self.next_request_id.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn send_request(&self, req: RequestPacket) -> io::Result<ResponsePacket> {
        let id = self.next_request_id();

        let (tx, rx) = oneshot::channel();

        self.pending_requests.lock().await.insert(id, tx);

        self.send_packet(Packet::Request {
            request_id: id,
            payload: req,
        })
        .await?;

        let response = rx
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::ConnectionAborted, "response dropped"))?;

        Ok(response)
    }
}
