use crate::structures::data_values::display_value::DisplayValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DisplayValues {
    display_values: Vec<DisplayValue>,
}

impl DisplayValues {
    pub fn new(display_values: Vec<DisplayValue>) -> Self {
        Self { display_values }
    }

    pub fn get_display_values(&self) -> &Vec<DisplayValue> {
        &self.display_values
    }

    pub fn get_display_value_string(
        &self,
        display_value_type: &DisplayValueType,
    ) -> &str {
        for display_value in &self.display_values {
            if display_value.get_display_value_type() == display_value_type {
                return display_value.get_display_value();
            }
        }

        "??"
    }

    pub fn get_display_value(
        &self,
        display_value_type: &DisplayValueType,
    ) -> Option<&DisplayValue> {
        for display_value in &self.display_values {
            if display_value.get_display_value_type() == display_value_type {
                return Some(display_value);
            }
        }

        None
    }
}
