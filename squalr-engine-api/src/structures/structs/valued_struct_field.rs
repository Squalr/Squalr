use std::{fmt, str::FromStr};

use crate::structures::data_values::data_value::DataValue;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ValuedStructFieldData {
    NestedStruct(Box<ValuedStructField>),
    Value(DataValue),
    Array(DataValue),
    Pointer32(u32),
    Pointer64(u64),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValuedStructField {
    name: String,
    field_data: ValuedStructFieldData,
}

impl ValuedStructField {
    pub fn new(
        name: String,
        field_data: ValuedStructFieldData,
    ) -> Self {
        Self { name, field_data }
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
            ValuedStructFieldData::Value(data_value) | ValuedStructFieldData::Array(data_value) => match data_value.get_default_display_value() {
                Some(display_value) => {
                    if pretty_print {
                        format!("{}{}\n", indent, display_value.get_display_value())
                    } else {
                        format!("{}{}", indent, display_value.get_display_value())
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

impl fmt::Display for ValuedStructField {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}", self.get_display_string(false, 0))
    }
}

impl FromStr for ValuedStructField {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts = input.splitn(2, ':');
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
                .map_err(|e| e.to_string())?
        } else {
            DataValue::from_str(value_str)
                .map(ValuedStructFieldData::Value)
                .map_err(|e| e.to_string())?
        };

        Ok(ValuedStructField::new(name, field_data))
    }
}
