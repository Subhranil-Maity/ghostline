use std::{fs, path::PathBuf};

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

use crate::crypto::{derive_peer_id, error::CryptoError};


/// The full local identity — private key lives here, never leaves this struct.
pub struct LocalIdentity {
    signing_key: SigningKey,
}

impl LocalIdentity {
    /// Generate a brand new random identity.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Load identity from disk, or generate and save if not found.
    pub fn load_or_generate(path: &PathBuf) -> Result<Self, CryptoError> {
        if path.exists() {
            Self::load(path)
        } else {
            let identity = Self::generate();
            identity.save(path)?;
            Ok(identity)
        }
    }

    /// Save the raw 32-byte private key seed to disk.
    /// Should ensure this file has restricted permissions (chmod 600).
    pub fn save(&self, path: &PathBuf) -> Result<(), CryptoError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        // store raw 32-byte secret seed
        fs::write(path, self.signing_key.to_bytes())?;

        // restrict permissions on unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Load a 32-byte private key seed from disk.
    pub fn load(path: &PathBuf) -> Result<Self, CryptoError> {
        let bytes = fs::read(path)?;
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKey(
                format!("expected 32 bytes, got {}", bytes.len())
            ));
        }
        let seed: [u8; 32] = bytes.try_into().unwrap();
        let signing_key = SigningKey::from_bytes(&seed);
        Ok(Self { signing_key })
    }

    /// The public verifying key — safe to share freely.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// The peer ID derived from the public key.
    /// First 16 bytes of public key, base58 encoded — short and stable.
    pub fn peer_id(&self) -> String {
        derive_peer_id(&self.signing_key.verifying_key())
    }

    /// Sign arbitrary bytes — useful for proving identity later.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Raw public key bytes — send these in the handshake packet.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }
}
