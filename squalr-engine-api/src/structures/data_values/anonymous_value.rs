use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents a value as a string that can potentially be converted to an explicit type later.
/// This is particularly useful when scannining for a value that may later be interpreted
/// as many data types, and supporting values passed via command line.
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnonymousValue {
    pub value_str: String,
    pub is_value_hex: bool,
}

impl AnonymousValue {
    pub fn new(
        value: &str,
        is_value_hex: bool,
    ) -> Self {
        AnonymousValue {
            value_str: value.to_string(),
            is_value_hex,
        }
    }
}

impl ToString for AnonymousValue {
    fn to_string(&self) -> String {
        self.value_str.clone()
    }
}

impl FromStr for AnonymousValue {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let is_value_hex = string.starts_with("0x");
        Ok(AnonymousValue::new(string, is_value_hex))
    }
}
