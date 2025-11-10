use crate::structures::{data_values::display_value_type::DisplayValueType, structs::container_type::ContainerType};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplayValue {
    display_string: String,
    display_value_type: DisplayValueType,
    container_type: ContainerType,
}

impl DisplayValue {
    pub fn new(
        display_string: String,
        display_value_type: DisplayValueType,
        container_type: ContainerType,
    ) -> Self {
        Self {
            display_string,
            display_value_type,
            container_type,
        }
    }

    pub fn get_display_value_type(&self) -> &DisplayValueType {
        &self.display_value_type
    }

    pub fn set_display_value_type(
        &mut self,
        display_value: DisplayValueType,
    ) {
        self.display_value_type = display_value
    }

    pub fn get_container_type(&self) -> &ContainerType {
        &self.container_type
    }

    pub fn get_display_string(&self) -> &str {
        &self.display_string
    }

    pub fn set_display_string(
        &mut self,
        display_string: String,
    ) {
        self.display_string = display_string;
    }
}

impl fmt::Display for DisplayValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}{}: {}", self.display_value_type, self.container_type, self.display_string)
    }
}
