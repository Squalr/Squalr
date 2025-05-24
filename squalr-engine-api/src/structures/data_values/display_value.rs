use crate::structures::data_values::display_value_type::DisplayValueType;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DisplayValue {
    display_value_type: DisplayValueType,
    display_value: String,
}

impl DisplayValue {
    pub fn new(
        display_value_type: DisplayValueType,
        display_value: String,
    ) -> Self {
        Self {
            display_value_type,
            display_value,
        }
    }

    pub fn get_display_value_type(&self) -> &DisplayValueType {
        &self.display_value_type
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
        write!(formatter, "{}: {}", self.display_value_type, self.display_value)
    }
}
