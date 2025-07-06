use crate::structures::{data_types::data_type_ref::DataTypeRef, structs::container_type::ContainerType};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolicStructFieldDefinition {
    data_type: DataTypeRef,
    container_type: ContainerType,
}

impl SymbolicStructFieldDefinition {
    pub fn new(
        data_type: DataTypeRef,
        container_type: ContainerType,
    ) -> Self {
        SymbolicStructFieldDefinition { data_type, container_type }
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.data_type.get_unit_size_in_bytes()
    }

    pub fn get_value(&self) -> &DataTypeRef {
        &self.data_type
    }
}

impl FromStr for SymbolicStructFieldDefinition {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // Determine container type based on string suffix.
        let (type_str, container_type) = if let Some(open_idx) = string.find('[') {
            if let Some(close_idx) = string.strip_suffix(']').map(|_| string.len() - 1) {
                let type_part = string[..open_idx].trim();
                let len_part = string[open_idx + 1..close_idx].trim();

                let len = len_part
                    .parse::<u64>()
                    .map_err(|error| format!("Invalid array length '{}': {}", len_part, error))?;

                (type_part, ContainerType::Array(len))
            } else {
                return Err("Missing closing ']' in array type".into());
            }
        } else if let Some(stripped) = string.strip_suffix('*') {
            (stripped, ContainerType::Pointer)
        } else {
            (string, ContainerType::None)
        };

        let data_type = DataTypeRef::from_str(type_str.trim())?;

        Ok(SymbolicStructFieldDefinition::new(data_type, container_type))
    }
}
