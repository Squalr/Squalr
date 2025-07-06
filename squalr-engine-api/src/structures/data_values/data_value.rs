use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::display_value::DisplayValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::structures::data_values::display_values::DisplayValues;
use crate::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use crate::structures::structs::valued_struct::ValuedStruct;
use crate::structures::structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData};
use crate::{registries::data_types::data_type_registry::DataTypeRegistry, structures::data_values::anonymous_value_container::AnonymousValueContainer};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug},
    mem,
    str::FromStr,
};

/// Represents a value for a `DataType`. Additionally, new `DataType` and `DataValue` pairs can be registered by plugins.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DataValue {
    /// The data type that this value represents.
    data_type_ref: DataTypeRef,

    /// The raw bytes of the data value. This could be a large number of underlying values, such as an int, string,
    /// or even a serialized bitfield and mask. It is the responsibility of the `DataType` object to interpret the bytes.
    value_bytes: Vec<u8>,

    /// The display values. These are created when the underlying value bytes change to prevent repeatedly allocating new strings when refreshing a value.
    display_values: DisplayValues,

    /// Override to the default display value.
    display_value_type_override: Option<DisplayValueType>,
}

impl DataValue {
    pub fn new(
        data_type_ref: DataTypeRef,
        value_bytes: Vec<u8>,
    ) -> Self {
        let display_values = Self::create_display_values(&data_type_ref, &value_bytes);

        Self {
            data_type_ref,
            value_bytes,
            display_values,
            display_value_type_override: None,
        }
    }

    pub fn copy_from_bytes(
        &mut self,
        value_bytes: &[u8],
    ) {
        // Only update the array and refresh the display value if the bytes are actually changed.
        if self.value_bytes != value_bytes {
            self.value_bytes = value_bytes.to_vec();
            self.display_values = Self::create_display_values(&self.data_type_ref, value_bytes);
        }
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.data_type_ref
    }

    pub fn set_data_type(
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

    pub fn get_display_values(&self) -> &DisplayValues {
        &self.display_values
    }

    pub fn get_display_value_type(&self) -> Option<DisplayValueType> {
        if let Some(display_value_type) = self.display_value_type_override {
            Some(display_value_type)
        } else if let Some(display_value) = self.display_values.get_default_display_value() {
            Some(*display_value.get_display_value_type())
        } else {
            None
        }
    }

    pub fn get_display_value(
        &self,
        display_value_type: &DisplayValueType,
    ) -> Option<DisplayValue> {
        for display_value in self.display_values.get_display_values() {
            if display_value.get_display_value_type() == display_value_type {
                return Some(display_value.clone());
            }
        }

        None
    }

    pub fn get_default_display_value(&self) -> Option<&DisplayValue> {
        self.display_values.get_default_display_value()
    }

    pub fn get_default_display_value_string(&self) -> &str {
        self.display_values.get_default_display_value_string()
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.value_bytes.as_ptr()
    }

    pub fn to_anonymous_valued_struct(&self) -> ValuedStruct {
        ValuedStruct::new(SymbolicStructRef::new_anonymous(), vec![self.to_anonymous_valued_struct_field()])
    }

    pub fn to_anonymous_valued_struct_field(&self) -> ValuedStructField {
        ValuedStructField::new(String::new(), ValuedStructFieldData::Value(self.clone()))
    }

    fn create_display_values(
        data_type_ref: &DataTypeRef,
        value_bytes: &[u8],
    ) -> DisplayValues {
        DataTypeRegistry::get_instance()
            .get(data_type_ref.get_data_type_id())
            .and_then(|data_type| data_type.create_display_values(value_bytes).ok())
            .unwrap_or_else(|| DisplayValues::new(vec![], DisplayValueType::String))
    }
}

impl FromStr for DataValue {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.split('=').collect();

        if parts.len() < 1 {
            return Err("Invalid data value string provided. Expected {data_type{;optional_container_size}}={value}".into());
        }

        let data_type_ref = DataTypeRef::from_str(parts[0])?;
        let anonymous_value_container = AnonymousValueContainer::from_str(parts[1])?;

        match anonymous_value_container.deanonymize_value(data_type_ref.get_data_type_id()) {
            Ok(value) => Ok(value),
            Err(err) => Err(format!("Unable to parse value: {}", err)),
        }
    }
}

impl fmt::Display for DataValue {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}={:?}", self.get_data_type_id(), self.get_display_values())
    }
}
