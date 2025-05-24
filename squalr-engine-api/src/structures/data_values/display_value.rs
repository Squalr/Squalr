use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DisplayValue {
    value_key: String,
    display_value: String,
}

impl DisplayValue {
    pub fn new(
        value_key: String,
        display_value: String,
    ) -> Self {
        Self { value_key, display_value }
    }

    pub fn get_value_key(&self) -> &str {
        &self.value_key
    }

    pub fn get_display_value(&self) -> &str {
        &self.display_value
    }
}

impl fmt::Display for DisplayValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}: {}", self.value_key, self.display_value)
    }
}
