use crate::structures::{data_values::display_value_type::DisplayValueType, structs::container_type::ContainerType};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DisplayValue {
    display_value: String,
    display_value_type: DisplayValueType,
    container_type: ContainerType,
}

impl DisplayValue {
    pub fn new(
        display_value: String,
        display_value_type: DisplayValueType,
        container_type: ContainerType,
    ) -> Self {
        Self {
            display_value,
            display_value_type,
            container_type,
        }
    }

    pub fn get_display_value_type(&self) -> &DisplayValueType {
        &self.display_value_type
    }

    pub fn get_container_type(&self) -> &ContainerType {
        &self.container_type
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
        write!(formatter, "{}{}: {}", self.display_value_type, self.container_type, self.display_value)
    }
}
