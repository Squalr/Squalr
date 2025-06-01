use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::data_values::anonymous_value::AnonymousValueContainer;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents an anonymous string value that can be further parsed into 1 to n anonymous values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnonymousValues {
    anonymous_values: Vec<AnonymousValue>,
}

impl AnonymousValues {
    pub fn new(
        value: &str,
        display_value_type: DisplayValueType,
    ) -> Self {
        AnonymousValues {
            anonymous_values: Self::parse_values(value, display_value_type),
        }
    }

    pub fn get_values(&self) -> &Vec<AnonymousValue> {
        &self.anonymous_values
    }
    pub fn deanonymize_value(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Result<DataValue, String> {
        for anonymous_value in self.get_values() {
            //
        }

        Err("No value available".to_string())
    }

    fn parse_values(
        value: &str,
        display_value_type: DisplayValueType,
    ) -> Vec<AnonymousValue> {
        match display_value_type.get_container_type() {
            ContainerType::Array => {
                // Split the input string into separate parts for the array
                value
                    .split(|character| character == ' ' || character == ',')
                    .filter(|anonymous_string| !anonymous_string.is_empty())
                    .map(|anonymous_value| {
                        let anonymous_value_string = anonymous_value.to_string();
                        let anonymous_value_format = match display_value_type {
                            DisplayValueType::Binary(_) => AnonymousValueContainer::BinaryValue(anonymous_value_string),
                            DisplayValueType::Hexadecimal(_) => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                            DisplayValueType::String(_) => AnonymousValueContainer::String(anonymous_value_string),
                            DisplayValueType::Bool(_) => AnonymousValueContainer::String(anonymous_value_string),
                            DisplayValueType::Decimal(_) => AnonymousValueContainer::String(anonymous_value_string),
                            DisplayValueType::Address(_) => AnonymousValueContainer::String(anonymous_value_string),
                            DisplayValueType::DataTypeRef(_) => AnonymousValueContainer::String(anonymous_value_string),
                            DisplayValueType::Enumeration(_) => AnonymousValueContainer::String(anonymous_value_string),
                        };

                        AnonymousValue::new(anonymous_value_format)
                    })
                    .collect()
            }
            ContainerType::None => vec![AnonymousValue::new(AnonymousValueContainer::String(
                value.to_string(),
            ))],
        }
    }
}

impl FromStr for AnonymousValues {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.rfind(';') {
            Some(pos) => {
                let (value_part, display_part) = string.split_at(pos);
                // Skip the ';'
                let display_part = &display_part[1..];

                // Try to parse DisplayValueType from the display_part
                let display_value_type = display_part
                    .parse::<DisplayValueType>()
                    .map_err(|err| format!("Failed to parse DisplayValueType: {}", err))?;

                Ok(AnonymousValues::new(value_part, display_value_type))
            }
            None => Err("Input string must take format of {value};{display_type}".to_string()),
        }
    }
}

impl fmt::Display for AnonymousValues {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let formatted_values: Vec<String> = self
            .anonymous_values
            .iter()
            .map(|value| format!("{}", value))
            .collect();

        write!(formatter, "{}", formatted_values.join(", "))
    }
}
