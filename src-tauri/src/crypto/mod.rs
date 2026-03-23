use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use std::fs;
use std::path::PathBuf;

use crate::crypto::error::CryptoError;

mod error;
mod local_identity;
pub use local_identity::LocalIdentity;
// pub use error;

// ─── Peer ID Derivation ───────────────────────────────────────────────────────

/// Derive a short stable peer ID from a public key.
/// Takes first 16 bytes of the 32-byte public key and base58 encodes them.
/// Output is ~22 characters, stable for the lifetime of the keypair.
pub fn derive_peer_id(verifying_key: &VerifyingKey) -> String {
    let bytes = verifying_key.to_bytes();
    bs58::encode(&bytes[..16]).into_string()
}

/// Derive peer ID directly from raw public key bytes received in a handshake.
pub fn peer_id_from_bytes(public_key_bytes: &[u8; 32]) -> Result<String, CryptoError> {
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
    Ok(derive_peer_id(&verifying_key))
}

// ─── Signature Verification ───────────────────────────────────────────────────

/// Verify a signature against a raw public key and message.
/// Useful later when peers sign their handshake to prove identity.
pub fn verify_signature(
    public_key_bytes: &[u8; 32],
    message: &[u8],
    signature_bytes: &[u8; 64],
) -> Result<bool, CryptoError> {
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;

    let signature = Signature::from_bytes(signature_bytes);

    match verifying_key.verify(message, &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// ─── Default Identity Path ────────────────────────────────────────────────────

/// Returns the default path for the identity key file.
/// ~/.ghostline/identity.key
pub fn default_identity_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".ghostline");
    path.push("identity.key");
    path
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::crypto::local_identity::LocalIdentity;

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_and_peer_id_stable() {
        let identity = LocalIdentity::generate();
        let id1 = identity.peer_id();
        let id2 = identity.peer_id();
        assert_eq!(id1, id2); // same identity = same peer_id always
        assert!(!id1.is_empty());
        println!("peer_id: {}", id1); // should be ~22 chars
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("identity.key");

        let original = LocalIdentity::generate();
        let original_id = original.peer_id();
        original.save(&path).unwrap();

        let loaded = LocalIdentity::load(&path).unwrap();
        let loaded_id = loaded.peer_id();

        assert_eq!(original_id, loaded_id); // survives disk roundtrip
    }

    #[test]
    fn test_load_or_generate_creates_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("identity.key");

        assert!(!path.exists());
        let identity = LocalIdentity::load_or_generate(&path).unwrap();
        assert!(path.exists());

        // loading again should give same peer_id
        let identity2 = LocalIdentity::load_or_generate(&path).unwrap();
        assert_eq!(identity.peer_id(), identity2.peer_id());
    }

    #[test]
    fn test_sign_and_verify() {
        let identity = LocalIdentity::generate();
        let message = b"ghostline handshake payload";

        let signature = identity.sign(message);
        let pub_key_bytes = identity.public_key_bytes();
        let sig_bytes: [u8; 64] = signature.to_bytes();

        let valid = verify_signature(&pub_key_bytes, message, &sig_bytes).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_peer_id_from_bytes_roundtrip() {
        let identity = LocalIdentity::generate();
        let pub_bytes = identity.public_key_bytes();
        let derived_id = peer_id_from_bytes(&pub_bytes).unwrap();
        assert_eq!(derived_id, identity.peer_id());
    }
}
