use crate::registries::data_types::data_type_registry::DataTypeRegistry;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::data_value::DataValue;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Internal enum to represent the two value modes.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnonymousValueContainer {
    String(String),
    BinaryValue(String),
    HexadecimalValue(String),
}

/// Represents a value as a string that can potentially be converted to an explicit type later.
/// This is particularly useful when scannining for a value such as `0`, which is valid across
/// many data types. This is helpful for supporting values passed via command line / GUI.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnonymousValue {
    anonymous_value_container: AnonymousValueContainer,
}

impl AnonymousValue {
    pub fn new(anonymous_value_container: AnonymousValueContainer) -> Self {
        AnonymousValue { anonymous_value_container }
    }

    pub fn get_value(&self) -> &AnonymousValueContainer {
        &self.anonymous_value_container
    }

    pub fn deanonymize_value(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Result<DataValue, String> {
        match DataTypeRegistry::get_instance().get(data_type_ref.get_data_type_id()) {
            Some(data_type) => {
                let deanonymized_value = data_type.deanonymize_value(&self, data_type_ref.clone());

                match deanonymized_value {
                    Ok(value) => Ok(value),
                    Err(err) => Err(err.to_string()),
                }
            }
            None => Err("Cannot deanonymize value: data type is not registered.".into()),
        }
    }
}

impl FromStr for AnonymousValue {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some((value, suffix)) = string.rsplit_once(';') {
            match suffix {
                "str" => Ok(AnonymousValue::new(AnonymousValueContainer::String(value.to_string()))),
                "bin" => Ok(AnonymousValue::new(AnonymousValueContainer::BinaryValue(value.to_string()))),
                "hex" => Ok(AnonymousValue::new(AnonymousValueContainer::HexadecimalValue(value.to_string()))),
                _ => Ok(AnonymousValue::new(AnonymousValueContainer::String(value.to_string()))),
            }
        } else {
            // No suffix — default to String
            Ok(AnonymousValue::new(AnonymousValueContainer::String(string.to_string())))
        }
    }
}

impl fmt::Display for AnonymousValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match &self.anonymous_value_container {
            AnonymousValueContainer::String(string) => {
                write!(formatter, "{};str", string)
            }
            AnonymousValueContainer::BinaryValue(string) => {
                write!(formatter, "{};bin", string)
            }
            AnonymousValueContainer::HexadecimalValue(string) => {
                write!(formatter, "{};hex", string)
            }
        }
    }
}
