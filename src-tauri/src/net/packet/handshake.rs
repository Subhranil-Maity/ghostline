use serde::{Deserialize, Serialize};

use crate::net::bytehandler::{
    error::PacketError,
    ByteReader, ByteWriter, Decode, Encode,
};

/// Handshake subtype
pub const HANDSHAKE_IDENTITY: u8 = 1;
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct PeerIdentity {
    // pub peer_id: String,        
    pub public_key_bytes: [u8; 32],
    pub display_name: String,  
    pub client_version: String, 
    pub capabilities: Vec<String>,
    pub timestamp: u64,        
}

impl Encode for PeerIdentity {
    fn encode(&self, w: &mut ByteWriter) {
        w.write_u8(HANDSHAKE_IDENTITY);
        // w.write_string(&self.peer_id);
        w.write_u8_array(&self.public_key_bytes);
        w.write_string(&self.display_name);
        w.write_string(&self.client_version);
        w.write_u32(self.capabilities.len() as u32);
        for capability in &self.capabilities {
            w.write_string(capability);
        }
        w.write_u64(self.timestamp);
    }
}

impl Decode for PeerIdentity {
    fn decode(r: &mut ByteReader) -> Result<Self, PacketError> {
        let subtype = r.read_u8()?;
        if subtype != HANDSHAKE_IDENTITY {
            return Err(PacketError::UnknownEventType(subtype));
        }

        // let peer_id = r.read_string()?;
        let public_key_bytes: [u8; 32] = r
            .read_u8_array()?
            .try_into()
            .map_err(|got: Vec<u8>| PacketError::UnexpectedEof {
                reading: "public_key_bytes",
                needed: 32,
                remaining: got.len(),
            })?;
        let display_name = r.read_string()?;
        let client_version = r.read_string()?;
        let cap_count = r.read_u32()? as usize;
        let mut capabilities = Vec::with_capacity(cap_count);
        for _ in 0..cap_count {
            capabilities.push(r.read_string()?);
        }
        let timestamp = r.read_u64()?;

        Ok(Self {
            public_key_bytes,
            display_name,
            client_version,
            capabilities,
            timestamp,
        })
    }
}
