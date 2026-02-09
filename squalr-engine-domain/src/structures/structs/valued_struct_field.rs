use crate::{structures::data_values::data_value::DataValue, traits::from_string_privileged::FromStringPrivileged};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValuedStructFieldData {
    Value(DataValue),
    NestedStruct(Box<ValuedStructField>),
}

impl Default for ValuedStructFieldData {
    fn default() -> Self {
        ValuedStructFieldData::Value(DataValue::default())
    }
}

/// Represents an editable display field
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValuedStructField {
    name: String,
    field_data: ValuedStructFieldData,
    is_read_only: bool,
}

impl ValuedStructField {
    pub fn new(
        name: String,
        field_data: ValuedStructFieldData,
        is_read_only: bool,
    ) -> Self {
        Self {
            name,
            field_data,
            is_read_only,
        }
    }

    pub fn get_data_value(&self) -> Option<&DataValue> {
        match &self.field_data {
            ValuedStructFieldData::Value(data_value) => Some(data_value),
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
        match &self.field_data {
            ValuedStructFieldData::NestedStruct(_nested_struct) => "",
            ValuedStructFieldData::Value(data_value) => data_value.get_data_type_id(),
        }
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

    /*
    pub fn get_data_value_interpreters(&self) -> Option<&DataValueInterpreters> {
        match &self.field_data {
            ValuedStructFieldData::NestedStruct(_nested_struct) => None,
            ValuedStructFieldData::Value(data_value) => Some(data_value.get_data_value_interpreters()),
        }
    }*/

    pub fn get_size_in_bytes(&self) -> u64 {
        match &self.field_data {
            ValuedStructFieldData::NestedStruct(nested_struct) => nested_struct.as_ref().get_size_in_bytes(),
            ValuedStructFieldData::Value(data_value) => data_value.get_size_in_bytes(),
        }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        match &self.field_data {
            ValuedStructFieldData::NestedStruct(nested_struct) => nested_struct.get_bytes(),
            ValuedStructFieldData::Value(data_value) => data_value.get_value_bytes().to_owned(),
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
            ValuedStructFieldData::Value(_data_value) => "TODO".to_string(), /*
                                                                             ValuedStructFieldData::Value(data_value) => match data_value.get_active_display_value() {
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
                                                                             }, */
        }
    }
}

impl<ContextType> FromStringPrivileged<ContextType> for ValuedStructField {
    type Err = String;

    fn from_string_privileged(
        string: &str,
        context: &ContextType,
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

        let field_data = DataValue::from_string_privileged(value_str, context)
            .map(ValuedStructFieldData::Value)
            .map_err(|error| error.to_string())?;

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
