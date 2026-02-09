use crate::{
    structures::{
        data_types::data_type_ref::DataTypeRef,
        structs::{
            symbolic_struct_ref::SymbolicStructRef,
            valued_struct::ValuedStruct,
            valued_struct_field::{ValuedStructField, ValuedStructFieldData},
        },
    },
    traits::from_string_privileged::FromStringPrivileged,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, mem, str::FromStr};

/// Represents a value for a `DataType`. Additionally, new `DataType` and `DataValue` pairs can be registered by plugins.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataValue {
    /// The data type that this value represents.
    data_type_ref: DataTypeRef,

    /// The raw bytes of the data value. This could be a large number of underlying values, such as an int, string,
    /// or even a serialized bitfield and mask. It is the responsibility of the `DataType` object to interpret the bytes.
    value_bytes: Vec<u8>,
}

impl DataValue {
    pub fn new(
        data_type_ref: DataTypeRef,
        value_bytes: Vec<u8>,
    ) -> Self {
        Self { data_type_ref, value_bytes }
    }

    pub fn copy_from_bytes(
        &mut self,
        value_bytes: &[u8],
    ) {
        // Only update the array and refresh the display value if the bytes are actually changed.
        if self.value_bytes != value_bytes {
            self.value_bytes = value_bytes.to_vec();
        }
    }

    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.data_type_ref
    }

    /// Updates the data type in place without updating the value bytes.
    pub fn set_data_type_in_place(
        &mut self,
        data_type_ref: DataTypeRef,
    ) {
        self.data_type_ref = data_type_ref;
    }

    pub fn get_data_type_id(&self) -> &str {
        &self.data_type_ref.get_data_type_id()
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.value_bytes.len() as u64
    }

    pub fn get_value_bytes(&self) -> &Vec<u8> {
        &self.value_bytes
    }

    pub fn take_value_bytes(&mut self) -> Vec<u8> {
        mem::take(&mut self.value_bytes)
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.value_bytes.as_ptr()
    }

    pub fn to_valued_struct(
        &self,
        is_read_only: bool,
    ) -> ValuedStruct {
        ValuedStruct::new(SymbolicStructRef::new_anonymous(), vec![self.to_valued_struct_field(is_read_only)])
    }

    pub fn to_valued_struct_field(
        &self,
        is_read_only: bool,
    ) -> ValuedStructField {
        ValuedStructField::new(String::new(), ValuedStructFieldData::Value(self.clone()), is_read_only)
    }

    pub fn to_named_valued_struct_field(
        &self,
        name: String,
        is_read_only: bool,
    ) -> ValuedStructField {
        ValuedStructField::new(name, ValuedStructFieldData::Value(self.clone()), is_read_only)
    }
}

impl<ContextType> FromStringPrivileged<ContextType> for DataValue {
    type Err = String;

    fn from_string_privileged(
        string: &str,
        _context: &ContextType,
    ) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.split('=').collect();

        if parts.len() < 1 {
            return Err("Invalid data value string provided. Expected {data_type{;optional_container_size}}={value}".into());
        }

        let data_type_ref = DataTypeRef::from_str(parts[0])?;
        let value_bytes = Vec::new();

        Ok(DataValue::new(data_type_ref, value_bytes))
    }
}
