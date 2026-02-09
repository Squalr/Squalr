use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use crate::structures::data_values::container_type::ContainerType;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents a value as a string that can potentially be converted to an explicit type later.
/// This is particularly useful when scannining for a value such as 0, which is valid across
/// many data types. This is helpful for supporting values passed via command line / GUI.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnonymousValueString {
    anonymous_value_string: String,
    anonymous_value_string_format: AnonymousValueStringFormat,
    container_type: ContainerType,
}

impl AnonymousValueString {
    pub fn new(
        anonymous_value_string: String,
        anonymous_value_string_format: AnonymousValueStringFormat,
        container_type: ContainerType,
    ) -> Self {
        AnonymousValueString {
            anonymous_value_string,
            anonymous_value_string_format,
            container_type,
        }
    }

    pub fn get_anonymous_value_string(&self) -> &str {
        &self.anonymous_value_string
    }

    pub fn set_anonymous_value_string(
        &mut self,
        new_string: String,
    ) {
        self.anonymous_value_string = new_string
    }

    pub fn get_anonymous_value_string_format(&self) -> AnonymousValueStringFormat {
        self.anonymous_value_string_format
    }

    pub fn set_anonymous_value_string_format(
        &mut self,
        new_format: AnonymousValueStringFormat,
    ) {
        self.anonymous_value_string_format = new_format
    }

    pub fn get_container_type(&self) -> ContainerType {
        self.container_type
    }

    pub fn set_container_type(
        &mut self,
        new_container_type: ContainerType,
    ) {
        self.container_type = new_container_type
    }
}

impl FromStr for AnonymousValueString {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.rsplitn(3, ';').collect();

        if parts.len() != 3 {
            return Err("Expected format: anonymous_string;anonymous_string_format;container_type".to_string());
        }

        let anonymous_value_string = parts[2].to_string();
        let anonymous_value_string_format = parts[1].parse::<AnonymousValueStringFormat>()?;
        let container_type = parts[0].parse::<ContainerType>()?;

        Ok(AnonymousValueString {
            anonymous_value_string,
            anonymous_value_string_format,
            container_type,
        })
    }
}

impl fmt::Display for AnonymousValueString {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            formatter,
            "{};{};{}",
            self.anonymous_value_string, self.anonymous_value_string_format, self.container_type
        )
    }
}
