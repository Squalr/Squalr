use crate::structures::data_values::data_value::DataValue;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    str::{FromStr, ParseBoolError},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Property {
    name: String,
    value: DataValue,
    is_read_only: bool,
}

impl Property {
    pub fn new(
        name: String,
        value: DataValue,
        is_read_only: bool,
    ) -> Self {
        Self { name, value, is_read_only }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self) -> &DataValue {
        &self.value
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
        let rest = parts[1];

        // Optional readonly flag
        let (value_str, is_read_only) = if let Some((value, flag)) = rest.rsplit_once(",readonly=") {
            let is_read_only = flag
                .trim()
                .parse::<bool>()
                .map_err(|err: ParseBoolError| err.to_string())?;
            (value.trim(), is_read_only)
        } else {
            (rest.trim(), false)
        };

        let value = DataValue::from_str(value_str).map_err(|err| format!("Invalid DataValue: {err}"))?;

        Ok(Property::new(name, value, is_read_only))
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
