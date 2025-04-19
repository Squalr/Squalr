use crate::structures::data_values::data_value::DataValue;
use crate::{registries::data_types::data_type_registry::DataTypeRegistry, structures::data_types::data_type_ref::DataTypeRef};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Internal enum to represent the two value modes.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnonymousValueContainer {
    StringValue(String, bool),
    ByteArray(Vec<u8>),
}

/// Represents a value as a string that can potentially be converted to an explicit type later.
/// This is particularly useful when scannining for a value such as `0`, which is valid across
/// many data types. This is helpful for supporting values passed via command line / GUI.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnonymousValue {
    pub value: AnonymousValueContainer,
}

impl AnonymousValue {
    pub fn new_string(
        value: &str,
        is_value_hex: bool,
    ) -> Self {
        AnonymousValue {
            value: AnonymousValueContainer::StringValue(value.to_string(), is_value_hex),
        }
    }

    pub fn new_bytes(bytes: Vec<u8>) -> Self {
        AnonymousValue {
            value: AnonymousValueContainer::ByteArray(bytes),
        }
    }

    pub fn deanonymize_value(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Result<DataValue, String> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(data_type_ref.get_data_type_id()) {
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

    pub fn get_value(&self) -> &AnonymousValueContainer {
        &self.value
    }
}

impl FromStr for AnonymousValue {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let is_value_hex = string.starts_with("0x");
        Ok(AnonymousValue::new_string(string, is_value_hex))
    }
}

impl fmt::Display for AnonymousValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match &self.value {
            AnonymousValueContainer::StringValue(string, is_hex) => {
                if *is_hex {
                    write!(formatter, "{}", string)
                } else {
                    write!(formatter, "{}", string)
                }
            }
            AnonymousValueContainer::ByteArray(bytes) => {
                write!(formatter, "{:?}", bytes)
            }
        }
    }
}
