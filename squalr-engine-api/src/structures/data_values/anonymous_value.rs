use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::{registries::data_types::data_type_registry::DataTypeRegistry, structures::data_types::data_type_ref::DataTypeRef};
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
    container_value: AnonymousValueContainer,
    container_type: ContainerType,
}

impl AnonymousValue {
    pub fn new(
        value: &str,
        display_value_type: DisplayValueType,
    ) -> Self {
        let (container_value, container_type) = match display_value_type {
            DisplayValueType::Bool(container_type) => (AnonymousValueContainer::String(value.to_string()), container_type),
            DisplayValueType::String(container_type) => (AnonymousValueContainer::String(value.to_string()), container_type),
            DisplayValueType::Binary(container_type) => (AnonymousValueContainer::BinaryValue(value.to_string()), container_type),
            DisplayValueType::Decimal(container_type) => (AnonymousValueContainer::String(value.to_string()), container_type),
            DisplayValueType::Hexadecimal(container_type) => (AnonymousValueContainer::HexadecimalValue(value.to_string()), container_type),
            DisplayValueType::Address(container_type) => (AnonymousValueContainer::String(value.to_string()), container_type),
            DisplayValueType::DataTypeRef(container_type) => (AnonymousValueContainer::String(value.to_string()), container_type),
            DisplayValueType::Enumeration(container_type) => (AnonymousValueContainer::String(value.to_string()), container_type),
        };

        AnonymousValue {
            container_value,
            container_type,
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
        // JIRA: Support parameters, container types, etc.
        Ok(AnonymousValue::new(string, DisplayValueType::String(ContainerType::None)))
    }
}

impl fmt::Display for AnonymousValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        // JIRA: container type
        match &self.container_value {
            AnonymousValueContainer::String(string) => {
                write!(formatter, "{}", string)
            }
            AnonymousValueContainer::BinaryValue(string) => {
                write!(formatter, "{}", string)
            }
            AnonymousValueContainer::HexadecimalValue(string) => {
                write!(formatter, "{}", string)
            }
        }
    }
}
