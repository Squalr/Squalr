use crate::structures::data_values::anonymous_value_container::AnonymousValueContainer;
use crate::structures::data_values::data_value::DataValue;
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
    pub fn new(
        value_string: &str,
        display_value: DisplayValue,
    ) -> Self {
        AnonymousValue {
            anonymous_value_container: Self::parse_anonymous_value(value_string, display_value),
        }
    }

    pub fn get_value(&self) -> &AnonymousValueContainer {
        &self.anonymous_value_container
    }

    pub fn deanonymize_value(
        &self,
        data_type_id: &str,
    ) -> Result<DataValue, String> {
        let deanonymized_values = self
            .anonymous_value_container
            .deanonymize_value(data_type_id)
            .map_err(|err: String| format!("Value deanonymization error: {:?}", err))?;

        Ok(deanonymized_values)
    }

    fn parse_anonymous_value(
        value_string: &str,
        display_value: DisplayValue,
    ) -> AnonymousValueContainer {
        match display_value.get_container_type() {
            ContainerType::Array => {
                // Split the input string into separate parts for the array.
                let anonymous_value_string = value_string.to_string();
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
            ContainerType::Pointer => {
                let data_value_string = value_string.to_string();

                match display_value.get_display_value_type() {
                    DisplayValueType::Binary => AnonymousValueContainer::BinaryValue(data_value_string),
                    DisplayValueType::Hexadecimal => AnonymousValueContainer::HexadecimalValue(data_value_string),
                    DisplayValueType::String => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Bool => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Decimal => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Address => AnonymousValueContainer::HexadecimalValue(data_value_string),
                    DisplayValueType::DataTypeRef => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Enumeration => AnonymousValueContainer::String(data_value_string),
                }
            }
            ContainerType::None => {
                let data_value_string = value_string.to_string();

                match display_value.get_display_value_type() {
                    DisplayValueType::Binary => AnonymousValueContainer::BinaryValue(data_value_string),
                    DisplayValueType::Hexadecimal => AnonymousValueContainer::HexadecimalValue(data_value_string),
                    DisplayValueType::String => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Bool => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Decimal => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Address => AnonymousValueContainer::HexadecimalValue(data_value_string),
                    DisplayValueType::DataTypeRef => AnonymousValueContainer::String(data_value_string),
                    DisplayValueType::Enumeration => AnonymousValueContainer::String(data_value_string),
                }
            }
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
