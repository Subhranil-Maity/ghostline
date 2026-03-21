pub type PacketResult<T> = Result<T, PacketError>;
#[derive(Debug)]
pub enum PacketError {
    UnexpectedEof {
        reading: &'static str,  // what field we were trying to read
        needed: usize,
        remaining: usize,
    },
    InvalidUtf8 {
        field: &'static str,
        source: std::string::FromUtf8Error,
    },
    UnknownPacketType(u8),
    UnknownEventType(u8),
    UnknownRequestType(u8),
    UnknownResponseType(u8),
    InvalidVersion {
        expected: u8,
        got: u8,
    },
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketError::UnexpectedEof { reading, needed, remaining } =>
                write!(f, "unexpected EOF while reading `{reading}`: needed {needed} bytes, had {remaining}"),
            PacketError::InvalidUtf8 { field, source } =>
                write!(f, "invalid UTF-8 in field `{field}`: {source}"),
            PacketError::UnknownPacketType(b) =>
                write!(f, "unknown packet type byte: 0x{b:02x}"),
            PacketError::UnknownEventType(b) =>
                write!(f, "unknown event subtype byte: 0x{b:02x}"),
            PacketError::UnknownRequestType(b) =>
                write!(f, "unknown request subtype byte: 0x{b:02x}"),
            PacketError::UnknownResponseType(b) =>
                write!(f, "unknown response subtype byte: 0x{b:02x}"),
            PacketError::InvalidVersion { expected, got } =>
                write!(f, "protocol version mismatch: expected {expected}, got {got}"),
        }
    }
}

impl std::error::Error for PacketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PacketError::InvalidUtf8 { source, .. } => Some(source),
            _ => None,
        }
    }
}
