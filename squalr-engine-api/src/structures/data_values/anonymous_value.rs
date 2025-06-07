use crate::registries::data_types::data_type_registry::DataTypeRegistry;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::data_value::DataValue;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::display_value_type::DisplayValueType;

/// Contains an individual part of an anonymous value.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
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
    anonymous_value_containers: Vec<AnonymousValueContainer>,
}

impl AnonymousValue {
    pub fn new(
        value_string: &str,
        display_value_type: DisplayValueType,
    ) -> Self {
        AnonymousValue {
            anonymous_value_containers: Self::parse_anonymous_value(value_string, display_value_type),
        }
    }

    pub fn get_values(&self) -> &Vec<AnonymousValueContainer> {
        &self.anonymous_value_containers
    }

    pub fn deanonymize_value(
        &self,
        data_type_id: &str,
    ) -> Result<DataValue, String> {
        match DataTypeRegistry::get_instance().get(data_type_id) {
            Some(data_type) => {
                let deanonymized_values = self
                    .anonymous_value_containers
                    .iter()
                    .map(|anonymous_value_container| {
                        data_type
                            .deanonymize_value(&anonymous_value_container)
                            .map_err(|err| format!("Value deanonymization error: {:?}", err))
                    })
                    .collect::<Result<Vec<_>, String>>()?;

                data_type
                    .array_merge(deanonymized_values)
                    .map_err(|err| format!("Value array merge error: {:?}", err))
            }
            None => Err("Cannot deanonymize value: data type is not registered.".into()),
        }
    }

    fn parse_anonymous_value(
        value_string: &str,
        display_value_type: DisplayValueType,
    ) -> Vec<AnonymousValueContainer> {
        match display_value_type.get_container_type() {
            ContainerType::Array => {
                // Split the input string into separate parts for the array
                value_string
                    .split(|character| character == ' ' || character == ',')
                    .filter(|anonymous_string| !anonymous_string.is_empty())
                    .map(|anonymous_value| {
                        let anonymous_value_string = anonymous_value.to_string();
                        let anonymous_value_container = match display_value_type {
                            DisplayValueType::Binary(_) => AnonymousValueContainer::BinaryValue(anonymous_value_string),
                            DisplayValueType::Hexadecimal(_) => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                            DisplayValueType::String(_) => AnonymousValueContainer::String(anonymous_value_string),
                            DisplayValueType::Bool(_) => AnonymousValueContainer::String(anonymous_value_string),
                            DisplayValueType::Decimal(_) => AnonymousValueContainer::String(anonymous_value_string),
                            DisplayValueType::Address(_) => AnonymousValueContainer::String(anonymous_value_string),
                            DisplayValueType::DataTypeRef(_) => AnonymousValueContainer::String(anonymous_value_string),
                            DisplayValueType::Enumeration(_) => AnonymousValueContainer::String(anonymous_value_string),
                        };

                        anonymous_value_container
                    })
                    .collect()
            }
            ContainerType::None => {
                let data_value_string = value_string.to_string();

                vec![match display_value_type {
                    DisplayValueType::Binary(_) => AnonymousValueContainer::BinaryValue(data_value_string),
                    DisplayValueType::Hexadecimal(_) => AnonymousValueContainer::HexadecimalValue(data_value_string),
                    DisplayValueType::String(_) => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Bool(_) => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Decimal(_) => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Address(_) => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::DataTypeRef(_) => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Enumeration(_) => AnonymousValueContainer::String(data_value_string),
                }]
            }
        }
    }
}

impl FromStr for AnonymousValue {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.rfind(';') {
            Some(pos) => {
                let (value_part, display_part) = string.split_at(pos);
                let display_part = &display_part[1..];

                // Try to parse DisplayValueType from the display_part
                let display_value_type = display_part
                    .parse::<DisplayValueType>()
                    .map_err(|err| format!("Failed to parse DisplayValueType: {}", err))?;

                Ok(AnonymousValue::new(value_part, display_value_type))
            }
            None => Err("Input string must take format of {value};{display_type}".to_string()),
        }
    }
}

impl fmt::Display for AnonymousValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let formatted_values: Vec<String> = self
            .anonymous_value_containers
            .iter()
            .map(|value| match value {
                AnonymousValueContainer::String(s) => format!("{}", s),
                AnonymousValueContainer::BinaryValue(s) => format!("{}", s),
                AnonymousValueContainer::HexadecimalValue(s) => format!("{}", s),
            })
            .collect();

        let suffix = match self.anonymous_value_containers.first() {
            Some(AnonymousValueContainer::String(_)) => "str",
            Some(AnonymousValueContainer::BinaryValue(_)) => "bin",
            Some(AnonymousValueContainer::HexadecimalValue(_)) => "hex",
            // Default / fallback to string.
            None => "str",
        };

        write!(formatter, "{};{}", formatted_values.join(", "), suffix)
    }
}
