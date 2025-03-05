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
    value: Vec<u8>,
}

impl DataValue {
    pub fn new(
        data_type: DataTypeRef,
        value: Vec<u8>,
    ) -> Self {
        Self { data_type, value }
    }

    pub fn copy_from_bytes(
        &mut self,
        value: &[u8],
    ) {
        self.value = value.to_vec()
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.value.len() as u64
    }

    pub fn get_value_bytes(&self) -> &Vec<u8> {
        &self.value
    }

    pub fn get_value_string(&self) -> String {
        let registry = DataTypeRegistry::get_instance().get_registry();

        String::new()
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.value.as_ptr()
    }
}
