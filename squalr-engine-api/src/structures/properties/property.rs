use crate::structures::structs::valued_struct::ValuedStruct;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Property {
    name: String,
    valued_struct: ValuedStruct,
    is_read_only: bool,
}

impl Property {
    pub fn new(
        name: String,
        valued_struct: ValuedStruct,
        is_read_only: bool,
    ) -> Self {
        Self {
            name,
            valued_struct,
            is_read_only,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_valued_struct(&self) -> &ValuedStruct {
        &self.valued_struct
    }

    pub fn get_display_string(
        &self,
        pretty_print: bool,
    ) -> String {
        self.valued_struct.get_display_string(pretty_print)
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
                        _ => return Err(format!("Unknown field: {key}")),
                    }
                }
            }
        }

        let value = ValuedStruct::from_str(value_str).map_err(|err| format!("Invalid DataValue: {err}"))?;

        Ok(Property::new(name, value, is_read_only))
    }
}

impl fmt::Display for Property {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        if self.is_read_only {
            write!(formatter, "{}={},readonly=true", self.name, self.valued_struct)
        } else {
            write!(formatter, "{}={}", self.name, self.valued_struct)
        }
    }
}
