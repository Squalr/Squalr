use crate::structures::data_values::data_value::DataValue;
use crate::structures::data_values::display_value_type::DisplayContainer;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::{registries::data_types::data_type_registry::DataTypeRegistry, structures::data_types::data_type_ref::DataTypeRef};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Internal enum to represent the two value modes.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnonymousValueContainer {
    StringValue(String),
    BinaryValue(String),
    HexValue(String),
    ByteArray(Vec<u8>),
}

/// Represents a value as a string that can potentially be converted to an explicit type later.
/// This is particularly useful when scannining for a value such as `0`, which is valid across
/// many data types. This is helpful for supporting values passed via command line / GUI.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnonymousValue {
    container_value: AnonymousValueContainer,
}

impl AnonymousValue {
    pub fn new(
        value: &str,
        display_value_type: DisplayValueType,
    ) -> Self {
        let container_value = match display_value_type {
            _ => AnonymousValueContainer::StringValue(value.to_string()),
        };

        AnonymousValue { container_value }
    }

    pub fn new_bytes(bytes: Vec<u8>) -> Self {
        AnonymousValue {
            container_value: AnonymousValueContainer::ByteArray(bytes),
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
        &self.container_value
    }
}

impl FromStr for AnonymousValue {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // JIRA: Support optional display type overrides.
        Ok(AnonymousValue::new(string, DisplayValueType::String(DisplayContainer::None)))
    }
}

impl fmt::Display for AnonymousValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match &self.container_value {
            AnonymousValueContainer::StringValue(string) => {
                write!(formatter, "{}", string)
            }
            AnonymousValueContainer::BinaryValue(string) => {
                write!(formatter, "{}", string)
            }
            AnonymousValueContainer::HexValue(string) => {
                write!(formatter, "{}", string)
            }
            AnonymousValueContainer::ByteArray(bytes) => {
                write!(formatter, "{:?}", bytes)
            }
        }
    }
}
