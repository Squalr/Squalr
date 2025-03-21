use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataTypeError {
    #[error("Failed to parse value: {0}")]
    ParseError(String),

    #[error("Invalid byte count: expected {expected} bytes, got {actual}")]
    InvalidByteCount { expected: usize, actual: usize },

    #[error("No bytes provided")]
    NoBytes,

    #[error("Invalid hex value '{hex}': {source}")]
    HexParseError {
        hex: String,
        #[source]
        source: std::num::ParseIntError,
    },
}
