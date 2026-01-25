use crate::{
    registries::{registries::Registries, symbols::symbol_registry::SymbolRegistry},
    structures::data_values::{data_value::DataValue, display_values::DisplayValues},
    traits::from_string_privileged::FromStringPrivileged,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValuedStructFieldData {
    Value(DataValue),
    Array(DataValue),
    Pointer32(u32),
    Pointer64(u64),
    NestedStruct(Box<ValuedStructField>),
}

impl Default for ValuedStructFieldData {
    fn default() -> Self {
        ValuedStructFieldData::Value(DataValue::default())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValuedStructField {
    name: String,
    field_data: ValuedStructFieldData,
    is_read_only: bool,
    icon_id: String,
}

impl ValuedStructField {
    pub fn new(
        name: String,
        field_data: ValuedStructFieldData,
        is_read_only: bool,
    ) -> Self {
        let symbol_registry = SymbolRegistry::get_instance();

        let icon_id = match &field_data {
            ValuedStructFieldData::Value(data_value) => symbol_registry.get_icon_id(data_value.get_data_type_ref()),
            ValuedStructFieldData::Array(data_value) => symbol_registry.get_icon_id(data_value.get_data_type_ref()),
            _ => "".to_string(),
        };

        Self {
            name,
            field_data,
            is_read_only,
            icon_id,
        }
    }

    pub fn get_data_value(&self) -> Option<&DataValue> {
        match &self.field_data {
            ValuedStructFieldData::Value(data_value) => Some(data_value),
            ValuedStructFieldData::Array(data_value) => Some(data_value),
            ValuedStructFieldData::Pointer32(_value) => None,
            ValuedStructFieldData::Pointer64(_value) => None,
            ValuedStructFieldData::NestedStruct(_nested_struct) => None,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_field_data(&self) -> &ValuedStructFieldData {
        &self.field_data
    }

    pub fn get_icon_id(&self) -> &str {
        &self.icon_id
    }

    pub fn set_field_data(
        &mut self,
        valued_field_data: ValuedStructFieldData,
    ) {
        self.field_data = valued_field_data;
    }

    pub fn get_is_read_only(&self) -> bool {
        self.is_read_only
    }

    pub fn get_display_values(&self) -> Option<&DisplayValues> {
        match &self.field_data {
            ValuedStructFieldData::NestedStruct(_nested_struct) => None,
            ValuedStructFieldData::Value(data_value) => Some(data_value.get_display_values()),
            ValuedStructFieldData::Array(data_value) => Some(data_value.get_display_values()),
            ValuedStructFieldData::Pointer32(_value) => None,
            ValuedStructFieldData::Pointer64(_value) => None,
        }
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        match &self.field_data {
            ValuedStructFieldData::NestedStruct(nested_struct) => nested_struct.as_ref().get_size_in_bytes(),
            ValuedStructFieldData::Value(data_value) => data_value.get_size_in_bytes(),
            ValuedStructFieldData::Array(data_value) => data_value.get_size_in_bytes(),
            ValuedStructFieldData::Pointer32(value) => size_of_val(value) as u64,
            ValuedStructFieldData::Pointer64(value) => size_of_val(value) as u64,
        }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        match &self.field_data {
            ValuedStructFieldData::NestedStruct(nested_struct) => nested_struct.get_bytes(),
            ValuedStructFieldData::Value(data_value) => data_value.get_value_bytes().to_owned(),
            ValuedStructFieldData::Array(data_value) => data_value.get_value_bytes().to_owned(),
            ValuedStructFieldData::Pointer32(value) => value.to_le_bytes().to_vec(),
            ValuedStructFieldData::Pointer64(value) => value.to_le_bytes().to_vec(),
        }
    }

    pub fn copy_from_bytes(
        &mut self,
        bytes: &[u8],
    ) {
        match &mut self.field_data {
            ValuedStructFieldData::NestedStruct(nested) => {
                nested.copy_from_bytes(bytes);
            }
            ValuedStructFieldData::Value(data_value) => {
                debug_assert!(bytes.len() as u64 >= data_value.get_size_in_bytes());

                data_value.copy_from_bytes(bytes);
            }
            ValuedStructFieldData::Array(data_value) => {
                debug_assert!(bytes.len() as u64 >= data_value.get_size_in_bytes());

                data_value.copy_from_bytes(bytes);
            }
            ValuedStructFieldData::Pointer32(value) => {
                debug_assert!(bytes.len() >= size_of_val(value));

                if bytes.len() >= size_of_val(value) {
                    *value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                }
            }
            ValuedStructFieldData::Pointer64(value) => {
                debug_assert!(bytes.len() >= size_of_val(value));

                if bytes.len() >= size_of_val(value) {
                    *value = u64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                    ]);
                }
            }
        }
    }

    pub fn get_display_string(
        &self,
        pretty_print: bool,
        tab_depth: i32,
    ) -> String {
        let indent = if pretty_print { "  ".repeat(tab_depth as usize) } else { String::new() };

        match &self.field_data {
            ValuedStructFieldData::NestedStruct(nested_struct) => {
                let nested_str = nested_struct
                    .as_ref()
                    .get_display_string(pretty_print, tab_depth.saturating_add(1));
                if pretty_print {
                    format!("{}{{\n{}\n{}}}", indent, nested_str, indent)
                } else {
                    format!("{{{}}}", nested_str)
                }
            }
            ValuedStructFieldData::Value(data_value) | ValuedStructFieldData::Array(data_value) => match data_value.get_active_display_value() {
                Some(display_value) => {
                    if pretty_print {
                        format!("{}{}\n", indent, display_value.get_display_string())
                    } else {
                        format!("{}{}", indent, display_value.get_display_string())
                    }
                }
                None => {
                    if pretty_print {
                        format!("{}\n", indent)
                    } else {
                        indent
                    }
                }
            },
            ValuedStructFieldData::Pointer64(value) => {
                if pretty_print {
                    format!("{}0x{:016X}\n", indent, value)
                } else {
                    format!("{}0x{:016X}", indent, value)
                }
            }
            ValuedStructFieldData::Pointer32(value) => {
                if pretty_print {
                    format!("{}0x{:08X}\n", indent, value)
                } else {
                    format!("{}0x{:08X}", indent, value)
                }
            }
        }
    }
}

impl FromStringPrivileged for ValuedStructField {
    type Err = String;

    fn from_string_privileged(
        string: &str,
        registries: &Registries,
    ) -> Result<Self, Self::Err> {
        let mut parts = string.splitn(2, ':');
        let name = parts
            .next()
            .ok_or_else(|| "Missing field name".to_string())?
            .trim()
            .to_string();
        let value_str = parts
            .next()
            .ok_or_else(|| "Missing field value".to_string())?
            .trim();

        let field_data = if value_str.starts_with("0x") {
            // JIRA: 32 bit support, explicitly, more explicit field type.
            u64::from_str_radix(&value_str[2..], 16)
                .map(ValuedStructFieldData::Pointer64)
                .map_err(|error| error.to_string())?
        } else {
            DataValue::from_string_privileged(value_str, registries)
                .map(ValuedStructFieldData::Value)
                .map_err(|error| error.to_string())?
        };

        let is_read_only = false;

        Ok(ValuedStructField::new(name, field_data, is_read_only))
    }
}

impl fmt::Display for ValuedStructField {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}", self.get_display_string(false, 0))
    }
}
