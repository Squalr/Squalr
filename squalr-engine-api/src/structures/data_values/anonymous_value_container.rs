use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Contains an individual part of an anonymous value.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AnonymousValueContainer {
    String(String),
    BinaryValue(String),
    HexadecimalValue(String),
}

impl FromStr for AnonymousValueContainer {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // First check for known suffixes.
        if let Some((value, suffix)) = string.rsplit_once('_') {
            match suffix.to_lowercase().as_str() {
                "bin" => return Ok(AnonymousValueContainer::BinaryValue(value.to_string())),
                "hex" => return Ok(AnonymousValueContainer::HexadecimalValue(value.to_string())),
                "str" => return Ok(AnonymousValueContainer::String(value.to_string())),
                _ => {}
            };
        }

        // If no suffix is provided, just interpret the value as a string.
        Ok(AnonymousValueContainer::String(string.to_string()))
    }
}

impl fmt::Display for AnonymousValueContainer {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            AnonymousValueContainer::String(val) => write!(formatter, "{}_str", val),
            AnonymousValueContainer::BinaryValue(val) => write!(formatter, "{}_bin", val),
            AnonymousValueContainer::HexadecimalValue(val) => write!(formatter, "{}_hex", val),
        }
    }
}
