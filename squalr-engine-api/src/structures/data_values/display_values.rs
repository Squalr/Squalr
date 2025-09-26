use crate::structures::data_values::display_value::DisplayValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplayValues {
    display_values: Vec<DisplayValue>,
    default_display_value_type: DisplayValueType,
    active_display_value_type: DisplayValueType,
    active_display_value_index: u64,
}

impl DisplayValues {
    pub fn new(
        display_values: Vec<DisplayValue>,
        default_display_value_type: DisplayValueType,
    ) -> Self {
        let active_display_value_type = default_display_value_type.clone();
        let active_display_value_index = display_values
            .iter()
            .position(|display_value| *display_value.get_display_value_type() == default_display_value_type)
            .unwrap_or(0) as u64;
        Self {
            display_values,
            default_display_value_type,
            active_display_value_type,
            active_display_value_index,
        }
    }

    pub fn set_active_display_value_type(
        &mut self,
        active_display_value_type: DisplayValueType,
    ) {
        self.active_display_value_type = active_display_value_type
    }

    pub fn get_active_display_value_type(&self) -> DisplayValueType {
        self.active_display_value_type
    }

    pub fn get_default_display_value_type(&self) -> DisplayValueType {
        self.default_display_value_type
    }

    pub fn set_active_display_value_index(
        &mut self,
        active_display_value_index: u64,
    ) {
        self.active_display_value_index = active_display_value_index
    }

    pub fn get_active_display_value_index(&self) -> u64 {
        self.active_display_value_index
    }

    pub fn get_display_values(&self) -> &Vec<DisplayValue> {
        &self.display_values
    }

    pub fn get_default_display_value_string(&self) -> &str {
        self.get_display_value_string(&self.default_display_value_type)
    }

    pub fn get_default_display_value(&self) -> Option<&DisplayValue> {
        self.get_display_value(&self.default_display_value_type)
    }

    pub fn get_active_display_value(&self) -> Option<&DisplayValue> {
        self.get_display_value(&self.active_display_value_type)
    }

    pub fn get_display_value_string(
        &self,
        display_value_type: &DisplayValueType,
    ) -> &str {
        for display_value in &self.display_values {
            if display_value.get_display_value_type() == display_value_type {
                return display_value.get_display_string();
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
