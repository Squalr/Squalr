use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::{container_type::ContainerType, data_value::DataValue, pointer_scan_pointer_size::PointerScanPointerSize},
    structs::{
        symbol_resolver::SymbolResolver,
        symbolic_expression::SymbolicExpression,
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
    #[serde(default, skip_serializing_if = "SymbolicFieldCountResolution::is_inferred")]
    count_resolution: SymbolicFieldCountResolution,
    #[serde(default, skip_serializing_if = "SymbolicFieldOffsetResolution::is_sequential")]
    offset_resolution: SymbolicFieldOffsetResolution,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolicFieldCountResolution {
    #[default]
    Inferred,
    Expression(SymbolicExpression),
}

impl SymbolicFieldCountResolution {
    pub fn new_expression(expression: SymbolicExpression) -> Self {
        Self::Expression(expression)
    }

    pub fn is_inferred(&self) -> bool {
        matches!(self, Self::Inferred)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolicFieldOffsetResolution {
    #[default]
    Sequential,
    Expression(SymbolicExpression),
}

impl SymbolicFieldOffsetResolution {
    pub fn new_expression(expression: SymbolicExpression) -> Self {
        Self::Expression(expression)
    }

    pub fn is_sequential(&self) -> bool {
        matches!(self, Self::Sequential)
    }
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
            count_resolution: SymbolicFieldCountResolution::Inferred,
            offset_resolution: SymbolicFieldOffsetResolution::Sequential,
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
            count_resolution: SymbolicFieldCountResolution::Inferred,
            offset_resolution: SymbolicFieldOffsetResolution::Sequential,
        }
    }

    pub fn new_named_with_resolutions(
        field_name: String,
        data_type_ref: DataTypeRef,
        container_type: ContainerType,
        count_resolution: SymbolicFieldCountResolution,
        offset_resolution: SymbolicFieldOffsetResolution,
    ) -> Self {
        SymbolicFieldDefinition {
            field_name,
            data_type_ref,
            container_type,
            count_resolution,
            offset_resolution,
        }
    }

    pub fn get_valued_struct_field(
        &self,
        symbol_registry: &impl SymbolResolver,
        is_read_only: bool,
    ) -> ValuedStructField {
        let field_data = if let Some(pointer_size) = self.container_type.get_pointer_size() {
            let default_value = symbol_registry
                .get_default_value(&pointer_size.to_data_type_ref())
                .unwrap_or_default();

            ValuedStructFieldData::Value(default_value)
        } else {
            match self.container_type {
                ContainerType::None => {
                    if let Some(default_value) = symbol_registry.get_default_value(&self.data_type_ref) {
                        ValuedStructFieldData::Value(default_value)
                    } else if let Some(nested_struct_layout) = symbol_registry.get_struct_layout(self.data_type_ref.get_data_type_id()) {
                        ValuedStructFieldData::NestedStruct(Box::new(nested_struct_layout.get_default_valued_struct(symbol_registry)))
                    } else {
                        ValuedStructFieldData::Value(DataValue::default())
                    }
                }
                ContainerType::Array => {
                    let array_value = self
                        .build_default_value_or_struct_bytes(symbol_registry, 1)
                        .unwrap_or_default();

                    ValuedStructFieldData::Value(array_value)
                }
                ContainerType::ArrayFixed(length) => {
                    let array_value = self
                        .build_default_value_or_struct_bytes(symbol_registry, length)
                        .unwrap_or_default();

                    ValuedStructFieldData::Value(array_value)
                }
                ContainerType::Pointer(_) | ContainerType::Pointer32 | ContainerType::Pointer64 => ValuedStructFieldData::Value(Default::default()),
            }
        };

        ValuedStructField::new(self.field_name.clone(), field_data, is_read_only)
    }

    fn build_default_value_or_struct_bytes(
        &self,
        symbol_registry: &impl SymbolResolver,
        element_count: u64,
    ) -> Option<DataValue> {
        let mut array_value = symbol_registry
            .get_default_value(&self.data_type_ref)
            .or_else(|| {
                let nested_struct_layout = symbol_registry.get_struct_layout(self.data_type_ref.get_data_type_id())?;
                let nested_struct_bytes = nested_struct_layout
                    .get_default_valued_struct(symbol_registry)
                    .get_bytes();

                Some(DataValue::new(self.data_type_ref.clone(), nested_struct_bytes))
            })?;
        let repeated_element_count = usize::try_from(element_count).ok()?;
        let repeated_bytes = array_value.get_value_bytes().repeat(repeated_element_count);

        array_value.copy_from_bytes(&repeated_bytes);

        Some(array_value)
    }

    pub fn get_size_in_bytes(
        &self,
        symbol_registry: &impl SymbolResolver,
    ) -> u64 {
        let unit_size_in_bytes = symbol_registry
            .get_default_value(&self.data_type_ref)
            .map(|default_value| default_value.get_size_in_bytes())
            .or_else(|| {
                symbol_registry
                    .get_struct_layout(self.data_type_ref.get_data_type_id())
                    .map(|nested_struct_layout| nested_struct_layout.get_size_in_bytes(symbol_registry))
            })
            .unwrap_or_else(|| symbol_registry.get_unit_size_in_bytes(&self.data_type_ref));

        self.container_type.get_total_size_in_bytes(unit_size_in_bytes)
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

    pub fn get_count_resolution(&self) -> &SymbolicFieldCountResolution {
        &self.count_resolution
    }

    pub fn get_offset_resolution(&self) -> &SymbolicFieldOffsetResolution {
        &self.offset_resolution
    }
}

impl FromStr for SymbolicFieldDefinition {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let trimmed_string = string.trim();
        let (field_definition_string, offset_resolution) = if let Some((field_definition_string, offset_expression_string)) = trimmed_string.split_once('@') {
            (
                field_definition_string.trim(),
                SymbolicFieldOffsetResolution::new_expression(SymbolicExpression::from_str(offset_expression_string.trim())?),
            )
        } else {
            (trimmed_string, SymbolicFieldOffsetResolution::Sequential)
        };
        let (field_name, type_and_container_string) = if let Some((field_name, type_and_container_string)) = field_definition_string.split_once(':') {
            let trimmed_field_name = field_name.trim();

            if trimmed_field_name.is_empty() {
                return Err("Missing field name before ':' in symbolic field definition.".to_string());
            }

            (trimmed_field_name.to_string(), type_and_container_string.trim())
        } else {
            (String::new(), field_definition_string)
        };

        let (type_str, container_type, count_resolution) = if let Some(open_bracket_position) = type_and_container_string.find('[') {
            if let Some(close_bracket_position) = type_and_container_string
                .strip_suffix(']')
                .map(|_| type_and_container_string.len() - 1)
            {
                let type_part = type_and_container_string[..open_bracket_position].trim();
                let length_part = type_and_container_string[open_bracket_position + 1..close_bracket_position].trim();

                if length_part.is_empty() {
                    (type_part, ContainerType::Array, SymbolicFieldCountResolution::Inferred)
                } else {
                    match length_part.parse::<u64>() {
                        Ok(array_length) => (type_part, ContainerType::ArrayFixed(array_length), SymbolicFieldCountResolution::Inferred),
                        Err(_) => (
                            type_part,
                            ContainerType::Array,
                            SymbolicFieldCountResolution::new_expression(SymbolicExpression::from_str(length_part)?),
                        ),
                    }
                }
            } else {
                return Err("Missing closing ']' in array type".into());
            }
        } else if type_and_container_string.ends_with(')') {
            if let Some(pointer_marker_index) = type_and_container_string.rfind("*(") {
                let type_part = type_and_container_string[..pointer_marker_index].trim();
                let container_type = ContainerType::from_str(&type_and_container_string[pointer_marker_index..])?;

                (type_part, container_type, SymbolicFieldCountResolution::Inferred)
            } else {
                (type_and_container_string, ContainerType::None, SymbolicFieldCountResolution::Inferred)
            }
        } else if let Some(stripped) = type_and_container_string.strip_suffix('*') {
            (
                stripped,
                ContainerType::from_pointer_size(PointerScanPointerSize::Pointer64),
                SymbolicFieldCountResolution::Inferred,
            )
        } else {
            (type_and_container_string, ContainerType::None, SymbolicFieldCountResolution::Inferred)
        };

        let data_type = DataTypeRef::from_str(type_str.trim())?;

        if field_name.is_empty() {
            Ok(SymbolicFieldDefinition {
                field_name,
                data_type_ref: data_type,
                container_type,
                count_resolution,
                offset_resolution,
            })
        } else {
            Ok(SymbolicFieldDefinition::new_named_with_resolutions(
                field_name,
                data_type,
                container_type,
                count_resolution,
                offset_resolution,
            ))
        }
    }
}

impl fmt::Display for SymbolicFieldDefinition {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let container_text = match &self.count_resolution {
            SymbolicFieldCountResolution::Inferred => self.container_type.to_string(),
            SymbolicFieldCountResolution::Expression(count_expression) => format!("[{}]", count_expression),
        };
        let field_text = if self.field_name.is_empty() {
            format!("{}{}", self.data_type_ref, container_text)
        } else {
            format!("{}:{}{}", self.field_name, self.data_type_ref, container_text)
        };

        if let SymbolicFieldOffsetResolution::Expression(offset_expression) = &self.offset_resolution {
            return write!(formatter, "{} @ {}", field_text, offset_expression);
        }

        write!(formatter, "{}", field_text)
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolicFieldCountResolution, SymbolicFieldDefinition, SymbolicFieldOffsetResolution};
    use crate::registries::symbols::{struct_layout_descriptor::StructLayoutDescriptor, symbol_registry::SymbolRegistry};
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{container_type::ContainerType, pointer_scan_pointer_size::PointerScanPointerSize},
        structs::{symbolic_expression::SymbolicExpression, symbolic_struct_definition::SymbolicStructDefinition, valued_struct_field::ValuedStructFieldData},
    };
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
    fn parse_extended_pointer_container_round_trips() {
        let symbolic_field_definition =
            SymbolicFieldDefinition::from_str("ptr:u8*(u24be)").expect("Expected extended pointer symbolic field definition to parse.");

        assert_eq!(
            symbolic_field_definition.get_container_type(),
            ContainerType::Pointer(PointerScanPointerSize::Pointer24be)
        );
        assert_eq!(symbolic_field_definition.to_string(), "ptr:u8*(u24be)");
    }

    #[test]
    fn parse_dynamic_array_count_expression_round_trips() {
        let symbolic_field_definition =
            SymbolicFieldDefinition::from_str("elements:game.Item[count] @ +8").expect("Expected dynamic array field definition to parse.");

        assert_eq!(symbolic_field_definition.get_field_name(), "elements");
        assert_eq!(symbolic_field_definition.get_data_type_ref(), &DataTypeRef::new("game.Item"));
        assert_eq!(symbolic_field_definition.get_container_type(), ContainerType::Array);
        assert_eq!(
            symbolic_field_definition.get_count_resolution(),
            &SymbolicFieldCountResolution::new_expression(SymbolicExpression::from_str("count").expect("Expected count expression to parse."))
        );
        assert_eq!(
            symbolic_field_definition.get_offset_resolution(),
            &SymbolicFieldOffsetResolution::new_expression(SymbolicExpression::from_str("+8").expect("Expected offset expression to parse."))
        );
        assert_eq!(symbolic_field_definition.to_string(), "elements:game.Item[count] @ +8");
    }

    #[test]
    fn parse_dynamic_array_expression_rejects_bad_syntax() {
        let parse_error = SymbolicFieldDefinition::from_str("elements:game.Item[count +] @ +8").expect_err("Expected dynamic array field definition to fail.");

        assert!(parse_error.contains("Expected expression value"));
    }

    #[test]
    fn get_valued_struct_field_preserves_named_field() {
        let symbol_registry = SymbolRegistry::new();
        let symbolic_field_definition = SymbolicFieldDefinition::new_named(String::from("position_x"), DataTypeRef::new("u16"), ContainerType::None);
        let valued_struct_field = symbolic_field_definition.get_valued_struct_field(&symbol_registry, false);

        assert_eq!(valued_struct_field.get_name(), "position_x");
    }

    #[test]
    fn get_valued_struct_field_materializes_nested_struct_fields() {
        let symbol_registry = SymbolRegistry::new();
        let nested_struct_layout = SymbolicStructDefinition::new(
            String::from("test.Nested"),
            vec![
                SymbolicFieldDefinition::new_named(String::from("low"), DataTypeRef::new("u16"), ContainerType::None),
                SymbolicFieldDefinition::new_named(String::from("high"), DataTypeRef::new("u32"), ContainerType::None),
            ],
        );
        symbol_registry.set_project_symbol_catalog(&[StructLayoutDescriptor::new(
            String::from("test.Nested"),
            nested_struct_layout,
        )]);
        let symbolic_field_definition = SymbolicFieldDefinition::new_named(String::from("Nested"), DataTypeRef::new("test.Nested"), ContainerType::None);
        let valued_struct_field = symbolic_field_definition.get_valued_struct_field(&symbol_registry, false);

        let ValuedStructFieldData::NestedStruct(nested_struct) = valued_struct_field.get_field_data() else {
            panic!("Expected nested struct field data.");
        };

        assert_eq!(nested_struct.get_fields().len(), 2);
        assert_eq!(nested_struct.get_size_in_bytes(), 6);
    }

    #[test]
    fn get_size_in_bytes_resolves_nested_struct_layout_size() {
        let symbol_registry = SymbolRegistry::new();
        let nested_struct_layout = SymbolicStructDefinition::new(
            String::from("test.Nested"),
            vec![
                SymbolicFieldDefinition::new_named(String::from("low"), DataTypeRef::new("u16"), ContainerType::None),
                SymbolicFieldDefinition::new_named(String::from("high"), DataTypeRef::new("u32"), ContainerType::None),
            ],
        );
        symbol_registry.set_project_symbol_catalog(&[StructLayoutDescriptor::new(
            String::from("test.Nested"),
            nested_struct_layout,
        )]);
        let symbolic_field_definition = SymbolicFieldDefinition::new(DataTypeRef::new("test.Nested"), ContainerType::ArrayFixed(3));

        assert_eq!(symbolic_field_definition.get_size_in_bytes(&symbol_registry), 18);
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
        assert_eq!(symbolic_field_definition.get_count_resolution(), &SymbolicFieldCountResolution::Inferred);
        assert_eq!(symbolic_field_definition.get_offset_resolution(), &SymbolicFieldOffsetResolution::Sequential);
    }
}
