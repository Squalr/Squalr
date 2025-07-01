use crate::structures::data_values::anonymous_value_container::AnonymousValueContainer;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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
    ) -> Result<Vec<DataValue>, String> {
        let deanonymized_values = self
            .anonymous_value_containers
            .iter()
            .map(|anonymous_value_container| {
                anonymous_value_container
                    .deanonymize_value(data_type_id)
                    .map_err(|err: String| format!("Value deanonymization error: {:?}", err))
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(deanonymized_values)
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
        let containers = string
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(|part| part.parse::<AnonymousValueContainer>())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AnonymousValue {
            anonymous_value_containers: containers,
        })
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
            .map(|anonymous_value_container| anonymous_value_container.to_string())
            .collect();

        write!(formatter, "{}", formatted_values.join(","))
    }
}
