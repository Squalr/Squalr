use crate::registries::data_types::data_type_registry::DataTypeRegistry;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug},
    str::FromStr,
};

use super::anonymous_value::AnonymousValue;

/// Represents a value for a `DataType`. Additionally, new `DataType` and `DataValue` pairs can be registered by plugins.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DataValue {
    /// A weak handle to the data type that this value represents.
    data_type: DataTypeRef,

    /// The raw bytes of the data value. This could be a large number of underlying values, such as an int, string,
    /// or even a serialized bitfield and mask. It is the responsibility of the `DataType` object to interpret the bytes.
    value_bytes: Vec<u8>,

    /// The display value. This is cached to prevent repeatedly allocating new strings when refreshing a value.
    display_value: String,
}

impl DataValue {
    pub fn new(
        data_type: DataTypeRef,
        value_bytes: Vec<u8>,
    ) -> Self {
        let display_value = Self::create_display_value(&data_type, &value_bytes);

        Self {
            data_type,
            value_bytes,
            display_value,
        }
    }

    pub fn copy_from_bytes(
        &mut self,
        value_bytes: &[u8],
    ) {
        // Only update the array and refresh the display value if the bytes are actually changed.
        if self.value_bytes != value_bytes {
            self.value_bytes = value_bytes.to_vec();
            self.display_value = Self::create_display_value(&self.data_type, value_bytes);
        }
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.data_type
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.value_bytes.len() as u64
    }

    pub fn get_value_bytes(&self) -> &Vec<u8> {
        &self.value_bytes
    }

    pub fn get_value_string(&self) -> &str {
        &self.display_value
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.value_bytes.as_ptr()
    }

    fn create_display_value(
        data_type: &DataTypeRef,
        value_bytes: &[u8],
    ) -> String {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(data_type.get_data_type_id()) {
            Some(data_type) => match data_type.create_display_value(value_bytes) {
                Ok(value_string) => value_string,
                Err(_) => "??".to_string(),
            },
            None => "??".to_string(),
        }
    }
}

impl FromStr for DataValue {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.split('=').collect();

        if parts.len() != 2 {
            return Err("Expected a format of {data_type_id}={value_string}".to_string());
        }

        match DataTypeRef::new(parts[0]) {
            Some(data_type) => {
                let is_value_hex = parts[1].starts_with("0x");
                let anonymous_value = AnonymousValue::new(parts[1], is_value_hex);

                match data_type.deanonymize_value(&anonymous_value) {
                    Ok(value) => Ok(value),
                    Err(err) => Err(format!("Unable to parse value: {}", err)),
                }
            }
            None => Err("Data type not found.".to_string()),
        }
    }
}

impl fmt::Display for DataValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}={}", self.get_data_type().get_data_type_id(), self.get_value_string())
    }
}
