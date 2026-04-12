use crate::structures::{
    data_types::{
        built_in_types::{u32::data_type_u32::DataTypeU32, u64::data_type_u64::DataTypeU64},
        data_type_ref::DataTypeRef,
    },
    data_values::container_type::ContainerType,
    structs::{
        symbol_resolver::SymbolResolver,
        valued_struct_field::{ValuedStructField, ValuedStructFieldData},
    },
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolicFieldDefinition {
    #[serde(default)]
    field_name: String,
    data_type_ref: DataTypeRef,
    container_type: ContainerType,
}

impl SymbolicFieldDefinition {
    pub fn new(
        data_type_ref: DataTypeRef,
        container_type: ContainerType,
    ) -> Self {
        SymbolicFieldDefinition {
            field_name: String::new(),
            data_type_ref,
            container_type,
        }
    }

    pub fn new_named(
        field_name: String,
        data_type_ref: DataTypeRef,
        container_type: ContainerType,
    ) -> Self {
        SymbolicFieldDefinition {
            field_name,
            data_type_ref,
            container_type,
        }
    }

    pub fn get_valued_struct_field(
        &self,
        symbol_registry: &impl SymbolResolver,
        is_read_only: bool,
    ) -> ValuedStructField {
        let field_data = match self.container_type {
            ContainerType::None => {
                let default_value = symbol_registry
                    .get_default_value(&self.data_type_ref)
                    .unwrap_or_default();

                ValuedStructFieldData::Value(default_value)
            }
            ContainerType::Pointer32 => {
                let default_value = symbol_registry
                    .get_default_value(&DataTypeRef::new(DataTypeU32::DATA_TYPE_ID))
                    .unwrap_or_default();

                ValuedStructFieldData::Value(default_value)
            }
            ContainerType::Pointer64 => {
                let default_value = symbol_registry
                    .get_default_value(&DataTypeRef::new(DataTypeU64::DATA_TYPE_ID))
                    .unwrap_or_default();

                ValuedStructFieldData::Value(default_value)
            }
            ContainerType::Array => {
                let array_value = symbol_registry
                    .get_default_value(&self.data_type_ref)
                    .unwrap_or_default();

                ValuedStructFieldData::Value(array_value)
            }
            ContainerType::ArrayFixed(length) => {
                let mut array_value = symbol_registry
                    .get_default_value(&self.data_type_ref)
                    .unwrap_or_default();
                let default_bytes = array_value.get_value_bytes();
                let repeated_bytes = default_bytes.repeat(length as usize);

                array_value.copy_from_bytes(&repeated_bytes);

                ValuedStructFieldData::Value(array_value)
            }
        };

        ValuedStructField::new(self.field_name.clone(), field_data, is_read_only)
    }

    pub fn get_size_in_bytes(
        &self,
        symbol_registry: &impl SymbolResolver,
    ) -> u64 {
        let unit_size_in_bytes = symbol_registry.get_unit_size_in_bytes(&self.data_type_ref);

        match self.container_type {
            ContainerType::None => unit_size_in_bytes,
            ContainerType::Pointer32 => 4,
            ContainerType::Pointer64 => 8,
            ContainerType::Array => unit_size_in_bytes,
            ContainerType::ArrayFixed(length) => unit_size_in_bytes.saturating_mul(length),
        }
    }

    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.data_type_ref
    }

    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    pub fn get_container_type(&self) -> ContainerType {
        self.container_type
    }
}

impl FromStr for SymbolicFieldDefinition {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let trimmed_string = string.trim();
        let (field_name, type_and_container_string) = if let Some((field_name, type_and_container_string)) = trimmed_string.split_once(':') {
            let trimmed_field_name = field_name.trim();

            if trimmed_field_name.is_empty() {
                return Err("Missing field name before ':' in symbolic field definition.".to_string());
            }

            (trimmed_field_name.to_string(), type_and_container_string.trim())
        } else {
            (String::new(), trimmed_string)
        };

        // Determine container type based on string suffix.
        let (type_str, container_type) = if let Some(open_idx) = type_and_container_string.find('[') {
            if let Some(close_idx) = type_and_container_string
                .strip_suffix(']')
                .map(|_| type_and_container_string.len() - 1)
            {
                let type_part = type_and_container_string[..open_idx].trim();
                let len_part = type_and_container_string[open_idx + 1..close_idx].trim();

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
        } else if let Some(stripped) = type_and_container_string.strip_suffix("*(32)") {
            (stripped, ContainerType::Pointer32)
        } else if let Some(stripped) = type_and_container_string.strip_suffix("*(64)") {
            (stripped, ContainerType::Pointer64)
        } else if let Some(stripped) = type_and_container_string.strip_suffix('*') {
            (stripped, ContainerType::Pointer64)
        } else {
            (type_and_container_string, ContainerType::None)
        };

        let data_type = DataTypeRef::from_str(type_str.trim())?;

        if field_name.is_empty() {
            Ok(SymbolicFieldDefinition::new(data_type, container_type))
        } else {
            Ok(SymbolicFieldDefinition::new_named(field_name, data_type, container_type))
        }
    }
}

impl fmt::Display for SymbolicFieldDefinition {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        if self.field_name.is_empty() {
            write!(formatter, "{}{}", self.data_type_ref, self.container_type)
        } else {
            write!(formatter, "{}:{}{}", self.field_name, self.data_type_ref, self.container_type)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolicFieldDefinition;
    use crate::registries::symbols::symbol_registry::SymbolRegistry;
    use crate::structures::{data_types::data_type_ref::DataTypeRef, data_values::container_type::ContainerType};
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn get_size_in_bytes_scales_fixed_arrays_by_element_count() {
        let symbol_registry = SymbolRegistry::new();
        let symbolic_field_definition = SymbolicFieldDefinition::new(DataTypeRef::new("u16"), ContainerType::ArrayFixed(3));

        assert_eq!(symbolic_field_definition.get_size_in_bytes(&symbol_registry), 6);
    }

    #[test]
    fn display_round_trips_base_type_and_container() {
        let symbolic_field_definition = SymbolicFieldDefinition::new(DataTypeRef::new("u8"), ContainerType::ArrayFixed(4));

        assert_eq!(symbolic_field_definition.to_string(), "u8[4]");
    }

    #[test]
    fn parse_named_field_round_trips_name_type_and_container() {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str("health:u32").expect("Expected named symbolic field definition to parse.");

        assert_eq!(symbolic_field_definition.get_field_name(), "health");
        assert_eq!(symbolic_field_definition.get_data_type_ref(), &DataTypeRef::new("u32"));
        assert_eq!(symbolic_field_definition.get_container_type(), ContainerType::None);
        assert_eq!(symbolic_field_definition.to_string(), "health:u32");
    }

    #[test]
    fn get_valued_struct_field_preserves_named_field() {
        let symbol_registry = SymbolRegistry::new();
        let symbolic_field_definition = SymbolicFieldDefinition::new_named(String::from("position_x"), DataTypeRef::new("u16"), ContainerType::None);
        let valued_struct_field = symbolic_field_definition.get_valued_struct_field(&symbol_registry, false);

        assert_eq!(valued_struct_field.get_name(), "position_x");
    }

    #[test]
    fn legacy_serialized_field_without_name_deserializes_as_anonymous_field() {
        let serialized_value = json!({
            "data_type_ref": { "data_type_id": "u32" },
            "container_type": "None"
        });
        let symbolic_field_definition: SymbolicFieldDefinition =
            serde_json::from_value(serialized_value).expect("Expected legacy symbolic field definition to deserialize.");

        assert_eq!(symbolic_field_definition.get_field_name(), "");
        assert_eq!(symbolic_field_definition.get_data_type_ref(), &DataTypeRef::new("u32"));
        assert_eq!(symbolic_field_definition.get_container_type(), ContainerType::None);
    }
}
