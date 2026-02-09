use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::container_type::ContainerType,
    structs::{
        symbol_resolver::SymbolResolver,
        valued_struct_field::{ValuedStructField, ValuedStructFieldData},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolicFieldDefinition {
    data_type_ref: DataTypeRef,
    container_type: ContainerType,
}

impl SymbolicFieldDefinition {
    pub fn new(
        data_type_ref: DataTypeRef,
        container_type: ContainerType,
    ) -> Self {
        SymbolicFieldDefinition { data_type_ref, container_type }
    }

    pub fn get_valued_struct_field(
        &self,
        symbol_registry: &Arc<RwLock<impl SymbolResolver>>,
        is_read_only: bool,
    ) -> ValuedStructField {
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return ValuedStructField::default();
            }
        };
        let field_data = match self.container_type {
            ContainerType::None => {
                let default_value = symbol_registry_guard
                    .get_default_value(&self.data_type_ref)
                    .unwrap_or_default();

                ValuedStructFieldData::Value(default_value)
            }
            ContainerType::Pointer32 => {
                let default_value = symbol_registry_guard
                    .get_default_value(&DataTypeRef::new("u32"))
                    .unwrap_or_default();

                ValuedStructFieldData::Value(default_value)
            }
            ContainerType::Pointer64 => {
                let default_value = symbol_registry_guard
                    .get_default_value(&DataTypeRef::new("u64"))
                    .unwrap_or_default();

                ValuedStructFieldData::Value(default_value)
            }
            ContainerType::Array => {
                let array_value = symbol_registry_guard
                    .get_default_value(&self.data_type_ref)
                    .unwrap_or_default();

                ValuedStructFieldData::Value(array_value)
            }
            ContainerType::ArrayFixed(length) => {
                let mut array_value = symbol_registry_guard
                    .get_default_value(&self.data_type_ref)
                    .unwrap_or_default();
                let default_bytes = array_value.get_value_bytes();
                let repeated_bytes = default_bytes.repeat(length as usize);

                array_value.copy_from_bytes(&repeated_bytes);

                ValuedStructFieldData::Value(array_value)
            }
        };

        ValuedStructField::new(String::new(), field_data, is_read_only)
    }

    pub fn get_size_in_bytes(
        &self,
        symbol_registry: &Arc<RwLock<impl SymbolResolver>>,
    ) -> u64 {
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return 0;
            }
        };
        symbol_registry_guard.get_unit_size_in_bytes(&self.data_type_ref)
    }

    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.data_type_ref
    }
}

impl FromStr for SymbolicFieldDefinition {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // Determine container type based on string suffix.
        let (type_str, container_type) = if let Some(open_idx) = string.find('[') {
            if let Some(close_idx) = string.strip_suffix(']').map(|_| string.len() - 1) {
                let type_part = string[..open_idx].trim();
                let len_part = string[open_idx + 1..close_idx].trim();

                if len_part.is_empty() {
                    // Arbitrary array: [].
                    (type_part, ContainerType::Array)
                } else {
                    // Fixed array: [length].
                    let array_length = len_part
                        .parse::<u64>()
                        .map_err(|error| format!("Invalid array length '{}': {}", len_part, error))?;

                    (type_part, ContainerType::ArrayFixed(array_length))
                }
            } else {
                return Err("Missing closing ']' in array type".into());
            }
        } else if let Some(stripped) = string.strip_suffix("*(32)") {
            (stripped, ContainerType::Pointer32)
        } else if let Some(stripped) = string.strip_suffix("*(64)") {
            (stripped, ContainerType::Pointer64)
        } else if let Some(stripped) = string.strip_suffix('*') {
            (stripped, ContainerType::Pointer64)
        } else {
            (string, ContainerType::None)
        };

        let data_type = DataTypeRef::from_str(type_str.trim())?;

        Ok(SymbolicFieldDefinition::new(data_type, container_type))
    }
}
