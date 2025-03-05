use crate::structures::{data_types::data_type_ref::DataTypeRef, registries::data_types::data_type_registry::DataTypeRegistry};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Represents a value for a `DataType`. Additionally, new `DataType` and `DataValue` pairs can be registered by plugins.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
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

        match registry.get(data_type.get_id()) {
            Some(data_type) => match data_type.create_display_value(value_bytes) {
                Some(value_string) => value_string,
                None => "??".to_string(),
            },
            None => "??".to_string(),
        }
    }
}
