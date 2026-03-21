pub mod event;
pub mod handshake;
pub mod request;
pub mod responce;

use serde::{Deserialize, Serialize};

use crate::net::bytehandler::{
    error::PacketError,
    ByteReader, ByteWriter, Decode, Encode,
};
use crate::net::packet::{
    event::EventPacket,
    handshake::PeerIdentity,
    request::RequestPacket,
    responce::ResponsePacket,
};

/// Packet type constants
const PACKET_EVENT: u8 = 1;
const PACKET_REQUEST: u8 = 2;
const PACKET_RESPONSE: u8 = 3;
const PACKET_HANDSHAKE: u8 = 4;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum Packet {
    Event(EventPacket),
    Request {
        request_id: u64,
        payload: RequestPacket,
    },
    Response {
        request_id: u64,
        payload: ResponsePacket,
    },
    HandShake(PeerIdentity),
}

impl Encode for Packet {
    fn encode(&self, w: &mut ByteWriter) {
        //version info
        w.write_u8(1);
        match self {
            Packet::Event(event) => {
                w.write_u8(PACKET_EVENT);
                event.encode(w);
            }
            Packet::Request {
                request_id,
                payload,
            } => {
                w.write_u8(PACKET_REQUEST);
                w.write_u64(*request_id);
                payload.encode(w);
            }
            Packet::Response {
                request_id,
                payload,
            } => {
                w.write_u8(PACKET_RESPONSE);
                w.write_u64(*request_id);
                payload.encode(w);
            }
            Packet::HandShake(peer_identity) => {
                w.write_u8(PACKET_HANDSHAKE);
                peer_identity.encode(w);
            }
        }
    }
}

impl Decode for Packet {
    fn decode(r: &mut ByteReader) -> Result<Self, PacketError> {
        let version = r.read_u8()?;
        // Hard Check for no reson
        if version != 1 {
            return Err(PacketError::InvalidVersion {
                expected: 1,
                got: version,
            });
        }

        match r.read_u8()? {
            PACKET_EVENT => Ok(Packet::Event(EventPacket::decode(r)?)),
            PACKET_REQUEST => Ok(Packet::Request {
                request_id: r.read_u64()?,
                payload: RequestPacket::decode(r)?,
            }),
            PACKET_RESPONSE => Ok(Packet::Response {
                request_id: r.read_u64()?,
                payload: ResponsePacket::decode(r)?,
            }),
            PACKET_HANDSHAKE => Ok(Packet::HandShake(PeerIdentity::decode(r)?)),
            other => Err(PacketError::UnknownPacketType(other)),
        }
    }
}

impl Packet {
    pub fn encode(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new();
        Encode::encode(self, &mut writer);
        writer.finish()
    }

    pub fn try_decode(buf: &[u8]) -> Result<Self, PacketError> {
        let mut reader = ByteReader::new(buf);
        Packet::decode(&mut reader)
    }
}

pub fn decode(buf: &[u8]) -> Option<Packet> {
    Packet::try_decode(buf).ok()
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
