use std::io;

use crate::{
    models::ChatMessage,
    net::{
        packet::{event::EventPacket, Packet},
        Connection, SIMPLE_TEXT_CHAT,
    },
};

pub async fn send_simple_text_packet(conn: &Connection, msg: ChatMessage) -> io::Result<()> {
    let caps = conn.connection_capabilities.lock().await;

    if !caps.contains(&SIMPLE_TEXT_CHAT.to_string()) {
        return Err(io::ErrorKind::Unsupported.into());
    }
    drop(caps); // release lock early

    let packet = Packet::Event(EventPacket::ChatMessage(msg.into()));
    Ok(conn.send_packet(packet).await?)
}
