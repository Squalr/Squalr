use crate::structures::data_values::container_type::ContainerType;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents a value as a vector of bytes that can potentially be converted to an explicit type later.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnonymousValueBytes {
    anonymous_value_bytes: Vec<u8>,
    container_type: ContainerType,
}

impl AnonymousValueBytes {
    pub fn new(
        anonymous_value_bytes: Vec<u8>,
        container_type: ContainerType,
    ) -> Self {
        Self {
            anonymous_value_bytes,
            container_type,
        }
    }

    pub fn get_value(&self) -> &[u8] {
        &self.anonymous_value_bytes
    }

    pub fn get_container_type(&self) -> ContainerType {
        self.container_type
    }
}

impl FromStr for AnonymousValueBytes {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.rsplitn(2, ';').collect();

        if parts.len() != 2 {
            return Err("Expected format: value;container_type".to_string());
        }

        let anonymous_value_bytes = parts[1].as_bytes().to_vec();
        let container_type = parts[0].parse::<ContainerType>()?;

        Ok(AnonymousValueBytes {
            anonymous_value_bytes,
            container_type,
        })
    }
}

impl fmt::Display for AnonymousValueBytes {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let value_str = String::from_utf8_lossy(&self.anonymous_value_bytes);
        write!(formatter, "{};{}", value_str, self.container_type)
    }
}
