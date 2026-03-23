
#[derive(Debug)]
pub enum CryptoError {
    Io(std::io::Error),
    InvalidKey(String),
    SignatureError(String),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::Io(e) => write!(f, "IO error: {}", e),
            CryptoError::InvalidKey(s) => write!(f, "Invalid key: {}", s),
            CryptoError::SignatureError(s) => write!(f, "Signature error: {}", s),
        }
    }
}

impl From<std::io::Error> for CryptoError {
    fn from(e: std::io::Error) -> Self {
        CryptoError::Io(e)
    }
}
