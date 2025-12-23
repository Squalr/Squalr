use crate::registries::registries::Registries;
use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_container::AnonymousValueContainer;
use crate::structures::data_values::display_value::DisplayValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::structures::data_values::display_values::DisplayValues;
use crate::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use crate::structures::structs::valued_struct::ValuedStruct;
use crate::structures::structs::valued_struct_field::{ValuedStructField, ValuedStructFieldNode};
use crate::traits::from_string_privileged::FromStringPrivileged;
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

    /// The display values for this data value.
    display_values: DisplayValues,
}

impl DataValue {
    pub fn new(
        data_type_ref: DataTypeRef,
        value_bytes: Vec<u8>,
    ) -> Self {
        let symbol_registry = SymbolRegistry::get_instance();
        let display_values = symbol_registry.create_display_values(&data_type_ref, &value_bytes);

        Self {
            data_type_ref,
            value_bytes,
            display_values,
        }
    }

    pub fn get_display_values(&self) -> &DisplayValues {
        &self.display_values
    }

    pub fn copy_from_bytes(
        &mut self,
        value_bytes: &[u8],
    ) {
        // Only update the array and refresh the display value if the bytes are actually changed.
        if self.value_bytes != value_bytes {
            self.value_bytes = value_bytes.to_vec();
        }

        let symbol_registry = SymbolRegistry::get_instance();
        self.display_values = symbol_registry.create_display_values(&self.data_type_ref, &value_bytes);
    }

    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.data_type_ref
    }

    pub fn set_data_type(
        &mut self,
        data_type_ref: DataTypeRef,
    ) {
        self.data_type_ref = data_type_ref;

        let symbol_registry = SymbolRegistry::get_instance();
        self.display_values = symbol_registry.create_display_values(&self.data_type_ref, &self.value_bytes);
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
        self.display_values = DisplayValues::default();

        mem::take(&mut self.value_bytes)
    }

    pub fn set_active_display_value_type(
        &mut self,
        active_display_value_type: DisplayValueType,
    ) {
        self.display_values
            .set_active_display_value_type(active_display_value_type);
    }

    pub fn get_active_display_value_type(&self) -> DisplayValueType {
        self.display_values.get_active_display_value_type()
    }

    pub fn get_active_display_value(&self) -> Option<&DisplayValue> {
        self.display_values.get_active_display_value()
    }

    pub fn get_default_display_value(&self) -> Option<&DisplayValue> {
        self.display_values.get_default_display_value()
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
        ValuedStructField::new(String::new(), ValuedStructFieldNode::Value(self.clone()), is_read_only)
    }

    pub fn to_named_valued_struct_field(
        &self,
        name: String,
        is_read_only: bool,
    ) -> ValuedStructField {
        ValuedStructField::new(name, ValuedStructFieldNode::Value(self.clone()), is_read_only)
    }
}

impl FromStringPrivileged for DataValue {
    type Err = String;

    fn from_string_privileged(
        string: &str,
        registries: &Registries,
    ) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.split('=').collect();

        if parts.len() < 1 {
            return Err("Invalid data value string provided. Expected {data_type{;optional_container_size}}={value}".into());
        }

        let data_type_ref = DataTypeRef::from_str(parts[0])?;
        let anonymous_value_container = AnonymousValueContainer::from_str(parts[1])?;
        let symbol_registry = registries.get_symbol_registry();
        let symbol_registry_guard = symbol_registry
            .read()
            .map_err(|error| format!("Failed to acquire read lock on SymbolRegistry: {}", error))?;

        match symbol_registry_guard.deanonymize_value(&data_type_ref, &anonymous_value_container) {
            Ok(value) => Ok(value),
            Err(error) => Err(format!("Unable to parse value: {}", error)),
        }
    }
}
