use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// pub enum Packet {
//     Event(EventPacket),
//
//     Request {
//         request_id: u64,
//         payload: RequestPacket,
//     },
//
//     Response {
//         request_id: u64,
//         payload: ResponsePacket,
//     },
// }

use std::io::Cursor;
use std::io::Read;

/// Packet type constants
const PACKET_EVENT: u8 = 1;
const PACKET_REQUEST: u8 = 2;
const PACKET_RESPONSE: u8 = 3;

/// Event subtypes
const EVENT_CHAT_MESSAGE: u8 = 1;

/// Request payload types
const REQ_GET_CAPABILITIES: u8 = 1;

/// Response payload types
const RES_CAPABILITIES: u8 = 1;
const RES_OK: u8 = 2;
const RES_ERROR: u8 = 3;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum EventPacket {
    ChatMessage(String),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum RequestPacket {
    GetCapabilities,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ResponsePacket {
    Capabilities { caps: Vec<String> },
    Ok,
    Error { message: String },
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum Packet {
    Event(EventPacket),
    Request { request_id: u64, payload: RequestPacket },
    Response { request_id: u64, payload: ResponsePacket },
}

impl Packet {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // version 
        buf.push(1);

        match self {
            Packet::Event(EventPacket::ChatMessage(msg)) => {
                buf.push(PACKET_EVENT); // packet type
                buf.push(EVENT_CHAT_MESSAGE); // event subtype
                let bytes = msg.as_bytes();
                // If The nomber Some time seems cheack for network byte order
                buf.extend((bytes.len() as u32).to_be_bytes());
                buf.extend(bytes);
            }

            Packet::Request { request_id, payload } => {
                buf.push(PACKET_REQUEST);
                buf.extend(request_id.to_be_bytes());

                match payload {
                    RequestPacket::GetCapabilities => {
                        buf.push(REQ_GET_CAPABILITIES);
                    }
                }
            }

            Packet::Response { request_id, payload } => {
                buf.push(PACKET_RESPONSE);
                buf.extend(request_id.to_be_bytes());

                match payload {
                    ResponsePacket::Capabilities { caps } => {
                        buf.push(RES_CAPABILITIES);
                        buf.extend((caps.len() as u32).to_be_bytes());
                        for c in caps {
                            let cb = c.as_bytes();
                            buf.extend((cb.len() as u32).to_be_bytes());
                            buf.extend(cb);
                        }
                    }
                    ResponsePacket::Ok => {
                        buf.push(RES_OK);
                    }
                    ResponsePacket::Error { message } => {
                        buf.push(RES_ERROR);
                        let mb = message.as_bytes();
                        buf.extend((mb.len() as u32).to_be_bytes());
                        buf.extend(mb);
                    }
                }
            }
        }

        buf
    }
}

/// Helper readers that return Option<T> for easy chaining on decode failures.
fn read_u8(cursor: &mut Cursor<&[u8]>) -> Option<u8> {
    let mut b = [0u8; 1];
    cursor.read_exact(&mut b).ok()?;
    Some(b[0])
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Option<u32> {
    let mut b = [0u8; 4];
    cursor.read_exact(&mut b).ok()?;
    Some(u32::from_be_bytes(b))
}

fn read_u64(cursor: &mut Cursor<&[u8]>) -> Option<u64> {
    let mut b = [0u8; 8];
    cursor.read_exact(&mut b).ok()?;
    Some(u64::from_be_bytes(b))
}

pub fn decode(buf: &[u8]) -> Option<Packet> {
    let mut cursor = Cursor::new(buf);
    let verion = read_u8(&mut cursor)?;
    if verion != 1 {
       panic!("The Version is for now 1 if It recives SOmething else then its wrong"); 
    }
    let packet_type = read_u8(&mut cursor)?;

    match packet_type {
        PACKET_EVENT => {
            // event subtype
            let subtype = read_u8(&mut cursor)?;
            match subtype {
                EVENT_CHAT_MESSAGE => {
                    let len = read_u32(&mut cursor)? as usize;
                    // protect against huge allocations
                    if len > (buf.len()) { return None; }

                    let mut msg = vec![0u8; len];
                    cursor.read_exact(&mut msg).ok()?;
                    let s = String::from_utf8(msg).ok()?;
                    Some(Packet::Event(EventPacket::ChatMessage(s)))
                }
                _ => None,
            }
        }

        PACKET_REQUEST => {
            let request_id = read_u64(&mut cursor)?;
            let payload_type = read_u8(&mut cursor)?;
            match payload_type {
                REQ_GET_CAPABILITIES => Some(Packet::Request {
                    request_id,
                    payload: RequestPacket::GetCapabilities,
                }),
                _ => None,
            }
        }

        PACKET_RESPONSE => {
            let request_id = read_u64(&mut cursor)?;
            let payload_type = read_u8(&mut cursor)?;
            match payload_type {
                RES_CAPABILITIES => {
                    let count = read_u32(&mut cursor)? as usize;
                    let mut caps = Vec::with_capacity(count);
                    for _ in 0..count {
                        let l = read_u32(&mut cursor)? as usize;
                        if l > buf.len() { return None; }
                        let mut b = vec![0u8; l];
                        cursor.read_exact(&mut b).ok()?;
                        caps.push(String::from_utf8(b).ok()?);
                    }
                    Some(Packet::Response {
                        request_id,
                        payload: ResponsePacket::Capabilities { caps },
                    })
                }
                RES_OK => Some(Packet::Response {
                    request_id,
                    payload: ResponsePacket::Ok,
                }),
                RES_ERROR => {
                    let len = read_u32(&mut cursor)? as usize;
                    if len > buf.len() { return None; }
                    let mut b = vec![0u8; len];
                    cursor.read_exact(&mut b).ok()?;
                    let message = String::from_utf8(b).ok()?;
                    Some(Packet::Response {
                        request_id,
                        payload: ResponsePacket::Error { message },
                    })
                }
                _ => None,
            }
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_roundtrip() {
        let p = Packet::Event(EventPacket::ChatMessage("hello world".to_string()));
        let enc = p.encode();
        let dec = decode(&enc).expect("should decode");
        assert_eq!(p, dec);
    }

    #[test]
    fn request_roundtrip() {
        let p = Packet::Request {
            request_id: 0x1122334455667788,
            payload: RequestPacket::GetCapabilities,
        };
        let enc = p.encode();
        let dec = decode(&enc).expect("should decode");
        assert_eq!(p, dec);
    }

    #[test]
    fn response_capabilities_roundtrip() {
        let p = Packet::Response {
            request_id: 42,
            payload: ResponsePacket::Capabilities {
                caps: vec!["a".into(), "b".into(), "rust".into()],
            },
        };
        let enc = p.encode();
        let dec = decode(&enc).expect("should decode");
        assert_eq!(p, dec);
    }

    #[test]
    fn response_ok_roundtrip() {
        let p = Packet::Response {
            request_id: 99,
            payload: ResponsePacket::Ok,
        };
        let enc = p.encode();
        let dec = decode(&enc).expect("should decode");
        assert_eq!(p, dec);
    }

    #[test]
    fn response_error_roundtrip() {
        let p = Packet::Response {
            request_id: 7,
            payload: ResponsePacket::Error {
                message: "oops".into(),
            },
        };
        let enc = p.encode();
        let dec = decode(&enc).expect("should decode");
        assert_eq!(p, dec);
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub enum EventPacket {
//     ChatMessage(String),
//
//     // UserTyping {
//     //     user: String,
//     // },
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub enum RequestPacket {
//     GetCapabilities,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub enum ResponsePacket {
//     Capabilities {
//         caps: Vec<String>,
//     },
//
//     Ok,
//
//     Error {
//         message: String,
//     },
// }
