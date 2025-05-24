use crate::structures::data_values::{data_value::DataValue, display_value_type::DisplayValueType};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Property {
    name: String,
    value: DataValue,
    is_read_only: bool,
    display_value_type: DisplayValueType,
}

impl Property {
    pub fn new(
        name: String,
        value: DataValue,
        is_read_only: bool,
        display_value_type: DisplayValueType,
    ) -> Self {
        Self {
            name,
            value,
            is_read_only,
            display_value_type,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self) -> &DataValue {
        &self.value
    }

    pub fn get_display_value(&self) -> &str {
        match &self
            .value
            .get_display_values()
            .get_display_value(&self.display_value_type)
        {
            Some(display_value) => display_value.get_display_value(),
            None => "??",
        }
    }

    pub fn get_is_read_only(&self) -> bool {
        self.is_read_only
    }
}

impl FromStr for Property {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split name and the rest.
        let parts: Vec<&str> = s.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err("Invalid format: missing '='".to_string());
        }

        let name = parts[0].trim().to_string();
        let rest = parts[1].trim();

        // Extract value before any optional metadata.
        let mut value_str = rest;
        let mut is_read_only = false;
        let mut display_value_type = DisplayValueType::String;

        // If there are additional fields, extract them.
        if let Some(index) = rest.find(",readonly=") {
            value_str = &rest[..index].trim();
            let metadata = &rest[index + 1..]; // skip the comma
            for field in metadata.split(',') {
                if let Some((key, value)) = field.split_once('=') {
                    match key {
                        "readonly" => {
                            is_read_only = value
                                .parse::<bool>()
                                .map_err(|err| format!("Invalid readonly flag: {err}"))?;
                        }
                        "display_value_type" => {
                            display_value_type = DisplayValueType::from_str(value).map_err(|_| format!("Invalid display_value_type: {value}"))?;
                        }
                        _ => return Err(format!("Unknown field: {key}")),
                    }
                }
            }
        }

        let value = DataValue::from_str(value_str).map_err(|err| format!("Invalid DataValue: {err}"))?;

        Ok(Property::new(name, value, is_read_only, display_value_type))
    }
}

impl fmt::Display for Property {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        if self.is_read_only {
            write!(formatter, "{}={},readonly=true", self.name, self.value)
        } else {
            write!(formatter, "{}={}", self.name, self.value)
        }
    }
}
