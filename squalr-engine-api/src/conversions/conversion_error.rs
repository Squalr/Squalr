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
    fn from(error: ParseIntError) -> Self {
        ConversionError::ParseError(error.to_string())
    }
}
