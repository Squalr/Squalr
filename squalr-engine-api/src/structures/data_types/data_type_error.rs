use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataTypeError {
    #[error("Failed to parse value: {0}")]
    ParseError(String),

    #[error("Invalid byte count: expected {expected} bytes, got {actual}")]
    InvalidByteCount { expected: usize, actual: usize },

    #[error("No bytes provided")]
    NoBytes,

    #[error("Invalid value '{value}', is_hex: {is_value_hex} => {source}")]
    ValueParseError {
        value: String,
        is_value_hex: bool,
        #[source]
        source: std::num::ParseIntError,
    },
}
