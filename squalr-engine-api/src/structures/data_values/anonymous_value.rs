use crate::structures::data_values::anonymous_value_container::AnonymousValueContainer;
use crate::structures::data_values::display_value::DisplayValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::structures::structs::container_type::ContainerType;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents a value as a string that can potentially be converted to an explicit type later.
/// This is particularly useful when scannining for a value such as `0`, which is valid across
/// many data types. This is helpful for supporting values passed via command line / GUI.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnonymousValue {
    anonymous_value_container: AnonymousValueContainer,
}

impl AnonymousValue {
    pub fn new(display_value: &DisplayValue) -> Self {
        AnonymousValue {
            anonymous_value_container: Self::parse_anonymous_value(display_value),
        }
    }

    pub fn get_value(&self) -> &AnonymousValueContainer {
        &self.anonymous_value_container
    }

    fn parse_anonymous_value(display_value: &DisplayValue) -> AnonymousValueContainer {
        let anonymous_value_string = display_value.get_display_string().to_string();

        match display_value.get_container_type() {
            ContainerType::Array(length) => {
                // Split the input string into separate parts for the array.
                let anonymous_value_container = match display_value.get_display_value_type() {
                    DisplayValueType::Binary => AnonymousValueContainer::BinaryValue(anonymous_value_string),
                    DisplayValueType::Hexadecimal => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                    DisplayValueType::String => AnonymousValueContainer::String(anonymous_value_string),
                    DisplayValueType::Bool => AnonymousValueContainer::String(anonymous_value_string),
                    DisplayValueType::Decimal => AnonymousValueContainer::String(anonymous_value_string),
                    DisplayValueType::Address => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                    DisplayValueType::DataTypeRef => AnonymousValueContainer::String(anonymous_value_string),
                    DisplayValueType::Enumeration => AnonymousValueContainer::String(anonymous_value_string),
                };

                anonymous_value_container
            }
            ContainerType::Pointer32 | ContainerType::Pointer64 => match display_value.get_display_value_type() {
                DisplayValueType::Binary => AnonymousValueContainer::BinaryValue(anonymous_value_string),
                DisplayValueType::Hexadecimal => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                DisplayValueType::String => AnonymousValueContainer::String(anonymous_value_string),
                DisplayValueType::Bool => AnonymousValueContainer::String(anonymous_value_string),
                DisplayValueType::Decimal => AnonymousValueContainer::String(anonymous_value_string),
                DisplayValueType::Address => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                DisplayValueType::DataTypeRef => AnonymousValueContainer::String(anonymous_value_string),
                DisplayValueType::Enumeration => AnonymousValueContainer::String(anonymous_value_string),
            },
            ContainerType::None => match display_value.get_display_value_type() {
                DisplayValueType::Binary => AnonymousValueContainer::BinaryValue(anonymous_value_string),
                DisplayValueType::Hexadecimal => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                DisplayValueType::String => AnonymousValueContainer::String(anonymous_value_string),
                DisplayValueType::Bool => AnonymousValueContainer::String(anonymous_value_string),
                DisplayValueType::Decimal => AnonymousValueContainer::String(anonymous_value_string),
                DisplayValueType::Address => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                DisplayValueType::DataTypeRef => AnonymousValueContainer::String(anonymous_value_string),
                DisplayValueType::Enumeration => AnonymousValueContainer::String(anonymous_value_string),
            },
        }
    }
}

impl FromStr for AnonymousValue {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let containers = string.parse::<AnonymousValueContainer>()?;

        Ok(AnonymousValue {
            anonymous_value_container: containers,
        })
    }
}

impl fmt::Display for AnonymousValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let formatted_value = self.anonymous_value_container.to_string();

        write!(formatter, "{}", formatted_value)
    }
}
