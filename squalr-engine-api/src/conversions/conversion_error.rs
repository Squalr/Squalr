use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Error parsing conversion")]
    ParseError(String),

    #[error("Unsupported conversion")]
    UnsupportedConversion,
}

impl From<ParseIntError> for ConversionError {
    fn from(err: ParseIntError) -> Self {
        ConversionError::ParseError(err.to_string())
    }
}
