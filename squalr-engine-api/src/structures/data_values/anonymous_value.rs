use crate::structures::data_values::anonymous_value_container::AnonymousValueContainer;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::data_value_interpretation_format::DataValueInterpretationFormat;
use crate::structures::data_values::data_value_interpreter::DataValueInterpreter;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents a value as a string that can potentially be converted to an explicit type later.
/// This is particularly useful when scannining for a value such as 0, which is valid across
/// many data types. This is helpful for supporting values passed via command line / GUI.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnonymousValue {
    anonymous_value_container: AnonymousValueContainer,
}

impl AnonymousValue {
    pub fn new(data_value_interpreter: &DataValueInterpreter) -> Self {
        AnonymousValue {
            anonymous_value_container: Self::parse_anonymous_value(data_value_interpreter),
        }
    }

    pub fn get_value(&self) -> &AnonymousValueContainer {
        &self.anonymous_value_container
    }

    fn parse_anonymous_value(data_value_interpreter: &DataValueInterpreter) -> AnonymousValueContainer {
        let anonymous_value_string = data_value_interpreter.get_display_string().to_string();

        match data_value_interpreter.get_container_type() {
            ContainerType::Array(length) => {
                // Split the input string into separate parts for the array.
                let anonymous_value_container = match data_value_interpreter.get_data_value_interpretation_format() {
                    DataValueInterpretationFormat::Binary => AnonymousValueContainer::BinaryValue(anonymous_value_string),
                    DataValueInterpretationFormat::Hexadecimal => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                    DataValueInterpretationFormat::String => AnonymousValueContainer::String(anonymous_value_string),
                    DataValueInterpretationFormat::Bool => AnonymousValueContainer::String(anonymous_value_string),
                    DataValueInterpretationFormat::Decimal => AnonymousValueContainer::String(anonymous_value_string),
                    DataValueInterpretationFormat::Address => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                    DataValueInterpretationFormat::DataTypeRef => AnonymousValueContainer::String(anonymous_value_string),
                    DataValueInterpretationFormat::Enumeration => AnonymousValueContainer::String(anonymous_value_string),
                };

                anonymous_value_container
            }
            ContainerType::Pointer32 | ContainerType::Pointer64 => match data_value_interpreter.get_data_value_interpretation_format() {
                DataValueInterpretationFormat::Binary => AnonymousValueContainer::BinaryValue(anonymous_value_string),
                DataValueInterpretationFormat::Hexadecimal => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                DataValueInterpretationFormat::String => AnonymousValueContainer::String(anonymous_value_string),
                DataValueInterpretationFormat::Bool => AnonymousValueContainer::String(anonymous_value_string),
                DataValueInterpretationFormat::Decimal => AnonymousValueContainer::String(anonymous_value_string),
                DataValueInterpretationFormat::Address => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                DataValueInterpretationFormat::DataTypeRef => AnonymousValueContainer::String(anonymous_value_string),
                DataValueInterpretationFormat::Enumeration => AnonymousValueContainer::String(anonymous_value_string),
            },
            ContainerType::None => match data_value_interpreter.get_data_value_interpretation_format() {
                DataValueInterpretationFormat::Binary => AnonymousValueContainer::BinaryValue(anonymous_value_string),
                DataValueInterpretationFormat::Hexadecimal => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                DataValueInterpretationFormat::String => AnonymousValueContainer::String(anonymous_value_string),
                DataValueInterpretationFormat::Bool => AnonymousValueContainer::String(anonymous_value_string),
                DataValueInterpretationFormat::Decimal => AnonymousValueContainer::String(anonymous_value_string),
                DataValueInterpretationFormat::Address => AnonymousValueContainer::HexadecimalValue(anonymous_value_string),
                DataValueInterpretationFormat::DataTypeRef => AnonymousValueContainer::String(anonymous_value_string),
                DataValueInterpretationFormat::Enumeration => AnonymousValueContainer::String(anonymous_value_string),
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
