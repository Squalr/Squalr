use thiserror::Error;

use crate::structures::data_values::container_type::ContainerType;

#[derive(Debug, Error)]
pub enum DataTypeError {
    #[error("Failed to parse value: {0}")]
    ParseError(String),

    #[error("Invalid byte count: expected {expected} bytes, got {actual}")]
    InvalidByteCount { expected: u64, actual: u64 },

    #[error("No bytes provided")]
    NoBytes,

    #[error("Invalid data type reference provided")]
    InvalidDataTypeRef { data_type_ref: String },

    #[error("Invalid data type reference provided")]
    UnsupportedContainerType { container_type: ContainerType },

    #[error("Invalid meta data")]
    InvalidMetaData,

    #[error("Unsupported display type")]
    UnsupportedDisplayType,

    #[error("Decoding error")]
    DecodingError { error: String },

    #[error("Data value merge error")]
    DataValueMergeError { error: String },

    #[error("Invalid value '{value}', is_hex: {is_value_hex} => {source}")]
    ValueParseError {
        value: String,
        is_value_hex: bool,
        #[source]
        source: std::num::ParseIntError,
    },
}
