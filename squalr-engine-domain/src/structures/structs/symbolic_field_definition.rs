use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::{container_type::ContainerType, data_value::DataValue, pointer_scan_pointer_size::PointerScanPointerSize},
    structs::{
        symbol_resolver::SymbolResolver,
        symbolic_resolver_definition::SymbolicResolverRef,
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
    #[serde(default, skip_serializing_if = "SymbolicFieldCountResolution::is_inferred")]
    display_count_resolution: SymbolicFieldCountResolution,
    #[serde(default, skip_serializing_if = "SymbolicFieldOffsetResolution::is_sequential")]
    offset_resolution: SymbolicFieldOffsetResolution,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active_when_resolver: Option<SymbolicResolverRef>,
    #[serde(default, skip_serializing_if = "is_false")]
    is_hidden: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolicFieldCountResolution {
    #[default]
    Inferred,
    Resolver(String),
}

impl SymbolicFieldCountResolution {
    pub fn new_resolver(resolver_id: String) -> Self {
        Self::Resolver(resolver_id)
    }

    pub fn is_inferred(&self) -> bool {
        matches!(self, Self::Inferred)
    }

    pub fn as_resolver_id(&self) -> Option<&str> {
        match self {
            Self::Resolver(resolver_id) => Some(resolver_id),
            Self::Inferred => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolicFieldOffsetResolution {
    #[default]
    Sequential,
    Static(u64),
    Resolver(String),
}

impl SymbolicFieldOffsetResolution {
    pub fn new_static(offset_in_bytes: u64) -> Self {
        Self::Static(offset_in_bytes)
    }

    pub fn new_resolver(resolver_id: String) -> Self {
        Self::Resolver(resolver_id)
    }

    pub fn is_sequential(&self) -> bool {
        matches!(self, Self::Sequential)
    }

    pub fn as_resolver_id(&self) -> Option<&str> {
        match self {
            Self::Resolver(resolver_id) => Some(resolver_id),
            Self::Sequential | Self::Static(_) => None,
        }
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
            display_count_resolution: SymbolicFieldCountResolution::Inferred,
            offset_resolution: SymbolicFieldOffsetResolution::Sequential,
            active_when_resolver: None,
            is_hidden: false,
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
            display_count_resolution: SymbolicFieldCountResolution::Inferred,
            offset_resolution: SymbolicFieldOffsetResolution::Sequential,
            active_when_resolver: None,
            is_hidden: false,
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
            display_count_resolution: SymbolicFieldCountResolution::Inferred,
            offset_resolution,
            active_when_resolver: None,
            is_hidden: false,
        }
    }

    pub fn new_named_with_resolutions_and_display_count(
        field_name: String,
        data_type_ref: DataTypeRef,
        container_type: ContainerType,
        count_resolution: SymbolicFieldCountResolution,
        display_count_resolution: SymbolicFieldCountResolution,
        offset_resolution: SymbolicFieldOffsetResolution,
    ) -> Self {
        SymbolicFieldDefinition {
            field_name,
            data_type_ref,
            container_type,
            count_resolution,
            display_count_resolution,
            offset_resolution,
            active_when_resolver: None,
            is_hidden: false,
        }
    }

    pub fn with_active_when_resolver(
        mut self,
        active_when_resolver: Option<SymbolicResolverRef>,
    ) -> Self {
        self.active_when_resolver = active_when_resolver;
        self
    }

    pub fn with_hidden(
        mut self,
        is_hidden: bool,
    ) -> Self {
        self.is_hidden = is_hidden;
        self
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
                ContainerType::PointerArray(pointer_size) => {
                    let default_value = symbol_registry
                        .get_default_value(&pointer_size.to_data_type_ref())
                        .unwrap_or_default();

                    ValuedStructFieldData::Value(default_value)
                }
                ContainerType::PointerArrayFixed(pointer_size, length) => {
                    let mut array_value = symbol_registry
                        .get_default_value(&pointer_size.to_data_type_ref())
                        .unwrap_or_default();
                    let repeated_element_count = usize::try_from(length).unwrap_or_default();
                    let repeated_bytes = array_value.get_value_bytes().repeat(repeated_element_count);

                    array_value.copy_from_bytes(&repeated_bytes);

                    ValuedStructFieldData::Value(array_value)
                }
                ContainerType::Pointer(_) => ValuedStructFieldData::Value(Default::default()),
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

    pub fn get_display_count_resolution(&self) -> &SymbolicFieldCountResolution {
        &self.display_count_resolution
    }

    pub fn get_offset_resolution(&self) -> &SymbolicFieldOffsetResolution {
        &self.offset_resolution
    }

    pub fn get_active_when_resolver(&self) -> Option<&SymbolicResolverRef> {
        self.active_when_resolver.as_ref()
    }

    pub fn is_hidden(&self) -> bool {
        self.is_hidden
    }
}

impl FromStr for SymbolicFieldDefinition {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let trimmed_string = string.trim();
        let (field_definition_string, offset_resolution) = if let Some((field_definition_string, offset_resolution_string)) = trimmed_string.split_once('@') {
            (field_definition_string.trim(), parse_offset_resolution(offset_resolution_string.trim())?)
        } else {
            (trimmed_string, SymbolicFieldOffsetResolution::Sequential)
        };
        let (field_definition_string, is_hidden) = parse_hidden_flag(field_definition_string);
        let (field_definition_string, active_when_resolver) = parse_active_when_resolver(field_definition_string)?;
        let (field_definition_string, display_count_resolution) = parse_display_count_resolution(field_definition_string)?;
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
                let (type_part, pointer_size) = split_pointer_suffix(type_part)?;

                if let Some(pointer_size) = pointer_size {
                    if length_part.is_empty() {
                        (type_part, ContainerType::PointerArray(pointer_size), SymbolicFieldCountResolution::Inferred)
                    } else {
                        match length_part.parse::<u64>() {
                            Ok(array_length) => (
                                type_part,
                                ContainerType::PointerArrayFixed(pointer_size, array_length),
                                SymbolicFieldCountResolution::Inferred,
                            ),
                            Err(_) => (type_part, ContainerType::PointerArray(pointer_size), parse_count_resolution(length_part)?),
                        }
                    }
                } else if length_part.is_empty() {
                    (type_part, ContainerType::Array, SymbolicFieldCountResolution::Inferred)
                } else {
                    match length_part.parse::<u64>() {
                        Ok(array_length) => (type_part, ContainerType::ArrayFixed(array_length), SymbolicFieldCountResolution::Inferred),
                        Err(_) => (type_part, ContainerType::Array, parse_count_resolution(length_part)?),
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
                display_count_resolution,
                offset_resolution,
                active_when_resolver,
                is_hidden,
            })
        } else {
            Ok(SymbolicFieldDefinition::new_named_with_resolutions_and_display_count(
                field_name,
                data_type,
                container_type,
                count_resolution,
                display_count_resolution,
                offset_resolution,
            )
            .with_active_when_resolver(active_when_resolver)
            .with_hidden(is_hidden))
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
            SymbolicFieldCountResolution::Resolver(resolver_id) => self.format_container_text_with_count(&format!("resolver({})", resolver_id)),
        };
        let mut field_text = if self.field_name.is_empty() {
            format!("{}{}", self.data_type_ref, container_text)
        } else {
            format!("{}:{}{}", self.field_name, self.data_type_ref, container_text)
        };

        match &self.display_count_resolution {
            SymbolicFieldCountResolution::Inferred => {}
            SymbolicFieldCountResolution::Resolver(resolver_id) => {
                field_text = format!("{} display resolver({})", field_text, resolver_id);
            }
        }

        if let Some(active_when_resolver) = self.active_when_resolver.as_ref() {
            field_text = format!("{} active resolver({})", field_text, active_when_resolver.get_resolver_id());
        }

        if self.is_hidden {
            field_text = format!("{} hidden", field_text);
        }

        match &self.offset_resolution {
            SymbolicFieldOffsetResolution::Static(offset_in_bytes) => return write!(formatter, "{} @ +{}", field_text, offset_in_bytes),
            SymbolicFieldOffsetResolution::Resolver(resolver_id) => return write!(formatter, "{} @ resolver({})", field_text, resolver_id),
            SymbolicFieldOffsetResolution::Sequential => {}
        }

        write!(formatter, "{}", field_text)
    }
}

impl SymbolicFieldDefinition {
    fn format_container_text_with_count(
        &self,
        count_text: &str,
    ) -> String {
        match self.container_type {
            ContainerType::PointerArray(pointer_size) | ContainerType::PointerArrayFixed(pointer_size, _) => {
                format!("*({})[{}]", pointer_size, count_text)
            }
            ContainerType::None | ContainerType::Array | ContainerType::ArrayFixed(_) | ContainerType::Pointer(_) => format!("[{}]", count_text),
        }
    }
}

fn split_pointer_suffix(type_part: &str) -> Result<(&str, Option<PointerScanPointerSize>), String> {
    if type_part.ends_with(')') {
        if let Some(pointer_marker_index) = type_part.rfind("*(") {
            let base_type_part = type_part[..pointer_marker_index].trim();
            let pointer_size = PointerScanPointerSize::from_str(&type_part[pointer_marker_index + 2..type_part.len() - 1])?;

            return Ok((base_type_part, Some(pointer_size)));
        }
    }

    if let Some(base_type_part) = type_part.strip_suffix('*') {
        return Ok((base_type_part.trim(), Some(PointerScanPointerSize::Pointer64)));
    }

    Ok((type_part, None))
}

fn parse_hidden_flag(field_definition_string: &str) -> (&str, bool) {
    let trimmed_field_definition_string = field_definition_string.trim();
    let Some(field_definition_without_hidden) = trimmed_field_definition_string.strip_suffix(" hidden") else {
        return (trimmed_field_definition_string, false);
    };

    (field_definition_without_hidden.trim(), true)
}

fn parse_active_when_resolver(field_definition_string: &str) -> Result<(&str, Option<SymbolicResolverRef>), String> {
    let trimmed_field_definition_string = field_definition_string.trim();
    let Some((field_definition_string, resolver_reference)) = trimmed_field_definition_string.rsplit_once(" active ") else {
        return Ok((trimmed_field_definition_string, None));
    };

    let active_when_resolver = parse_resolver_reference(resolver_reference)
        .and_then(SymbolicResolverRef::new)
        .ok_or_else(|| String::from("Active variant resolver must use resolver(...)."))?;

    Ok((field_definition_string.trim(), Some(active_when_resolver)))
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn parse_count_resolution(length_part: &str) -> Result<SymbolicFieldCountResolution, String> {
    if let Some(resolver_id) = parse_resolver_reference(length_part) {
        return Ok(SymbolicFieldCountResolution::new_resolver(resolver_id));
    }

    Err(format!("Dynamic array count must use resolver(...), got `{}`.", length_part))
}

fn parse_offset_resolution(offset_part: &str) -> Result<SymbolicFieldOffsetResolution, String> {
    if let Some(resolver_id) = parse_resolver_reference(offset_part) {
        return Ok(SymbolicFieldOffsetResolution::new_resolver(resolver_id));
    }

    parse_static_offset(offset_part).map(SymbolicFieldOffsetResolution::new_static)
}

fn parse_display_count_resolution(field_definition_string: &str) -> Result<(&str, SymbolicFieldCountResolution), String> {
    let Some((field_definition_string, display_count_string)) = field_definition_string.rsplit_once(" display ") else {
        return Ok((field_definition_string, SymbolicFieldCountResolution::Inferred));
    };
    let trimmed_display_count_string = display_count_string.trim();
    if trimmed_display_count_string.is_empty() {
        return Err(String::from("Display count resolver is required."));
    }

    Ok((field_definition_string.trim(), parse_count_resolution(trimmed_display_count_string)?))
}

fn parse_static_offset(offset_part: &str) -> Result<u64, String> {
    let offset_part = offset_part.trim();
    let offset_part = offset_part
        .strip_prefix('+')
        .map(str::trim)
        .unwrap_or(offset_part);
    let offset_binary_part = offset_part
        .strip_prefix("0b")
        .or_else(|| offset_part.strip_prefix("0B"));
    let offset_hex_part = offset_part
        .strip_prefix("0x")
        .or_else(|| offset_part.strip_prefix("0X"));

    if let Some(offset_binary_part) = offset_binary_part {
        u64::from_str_radix(offset_binary_part, 2).map_err(|_| format!("Invalid static field offset: {}.", offset_part))
    } else if let Some(offset_hex_part) = offset_hex_part {
        u64::from_str_radix(offset_hex_part, 16).map_err(|_| format!("Invalid static field offset: {}.", offset_part))
    } else {
        offset_part
            .parse::<u64>()
            .map_err(|_| format!("Field offset must be a static unsigned value or resolver(...), got `{}`.", offset_part))
    }
}

fn parse_resolver_reference(resolver_reference: &str) -> Option<String> {
    let resolver_id = resolver_reference
        .strip_prefix("resolver(")?
        .strip_suffix(')')?
        .trim();

    (!resolver_id.is_empty()).then_some(resolver_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::{SymbolicFieldCountResolution, SymbolicFieldDefinition, SymbolicFieldOffsetResolution};
    use crate::registries::symbols::{struct_layout_descriptor::StructLayoutDescriptor, symbol_registry::SymbolRegistry};
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{container_type::ContainerType, pointer_scan_pointer_size::PointerScanPointerSize},
        structs::{symbolic_struct_definition::SymbolicStructDefinition, valued_struct_field::ValuedStructFieldData},
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
    fn parse_fixed_pointer_array_round_trips() {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str("entities:Entity*(u64)[1024] display resolver(entity.count)")
            .expect("Expected fixed pointer array field definition to parse.");

        assert_eq!(symbolic_field_definition.get_field_name(), "entities");
        assert_eq!(symbolic_field_definition.get_data_type_ref(), &DataTypeRef::new("Entity"));
        assert_eq!(
            symbolic_field_definition.get_container_type(),
            ContainerType::PointerArrayFixed(PointerScanPointerSize::Pointer64, 1024)
        );
        assert_eq!(
            symbolic_field_definition.get_display_count_resolution(),
            &SymbolicFieldCountResolution::new_resolver(String::from("entity.count"))
        );
        assert_eq!(
            symbolic_field_definition.to_string(),
            "entities:Entity*(u64)[1024] display resolver(entity.count)"
        );
    }

    #[test]
    fn parse_dynamic_pointer_array_round_trips() {
        let symbolic_field_definition =
            SymbolicFieldDefinition::from_str("entities:Entity*(u32)[resolver(entity.count)]").expect("Expected pointer array field definition to parse.");

        assert_eq!(
            symbolic_field_definition.get_container_type(),
            ContainerType::PointerArray(PointerScanPointerSize::Pointer32)
        );
        assert_eq!(
            symbolic_field_definition.get_count_resolution(),
            &SymbolicFieldCountResolution::new_resolver(String::from("entity.count"))
        );
        assert_eq!(symbolic_field_definition.to_string(), "entities:Entity*(u32)[resolver(entity.count)]");
    }

    #[test]
    fn parse_dynamic_array_count_requires_resolver() {
        let parse_error = SymbolicFieldDefinition::from_str("elements:game.Item[count] @ +8").expect_err("Expected inline count expression to fail.");

        assert!(parse_error.contains("Dynamic array count must use resolver"));
    }

    #[test]
    fn parse_static_field_offset_round_trips() {
        let symbolic_field_definition =
            SymbolicFieldDefinition::from_str("elements:game.Item[resolver(game.item_count)] @ +8").expect("Expected static offset to parse.");

        assert_eq!(symbolic_field_definition.get_field_name(), "elements");
        assert_eq!(symbolic_field_definition.get_data_type_ref(), &DataTypeRef::new("game.Item"));
        assert_eq!(symbolic_field_definition.get_container_type(), ContainerType::Array);
        assert_eq!(
            symbolic_field_definition.get_count_resolution(),
            &SymbolicFieldCountResolution::new_resolver(String::from("game.item_count"))
        );
        assert_eq!(symbolic_field_definition.get_offset_resolution(), &SymbolicFieldOffsetResolution::new_static(8));
        assert_eq!(symbolic_field_definition.to_string(), "elements:game.Item[resolver(game.item_count)] @ +8");

        let hex_offset_field = SymbolicFieldDefinition::from_str("field:u8 @ +0x10").expect("Expected hex static offset to parse.");
        let binary_offset_field = SymbolicFieldDefinition::from_str("field:u8 @ +0b10000").expect("Expected binary static offset to parse.");

        assert_eq!(hex_offset_field.get_offset_resolution(), &SymbolicFieldOffsetResolution::new_static(16));
        assert_eq!(binary_offset_field.get_offset_resolution(), &SymbolicFieldOffsetResolution::new_static(16));
    }

    #[test]
    fn parse_dynamic_array_resolver_reference_round_trips() {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str("elements:game.Item[resolver(game.item_count)] @ resolver(game.item_offset)")
            .expect("Expected dynamic array resolver field definition to parse.");

        assert_eq!(
            symbolic_field_definition.get_count_resolution(),
            &SymbolicFieldCountResolution::new_resolver(String::from("game.item_count"))
        );
        assert_eq!(
            symbolic_field_definition.get_offset_resolution(),
            &SymbolicFieldOffsetResolution::new_resolver(String::from("game.item_offset"))
        );
        assert_eq!(
            symbolic_field_definition.to_string(),
            "elements:game.Item[resolver(game.item_count)] @ resolver(game.item_offset)"
        );
    }

    #[test]
    fn parse_fixed_array_display_count_resolver_round_trips() {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str("entities:u64[1024] display resolver(entity.count)")
            .expect("Expected fixed array display resolver field definition to parse.");

        assert_eq!(symbolic_field_definition.get_field_name(), "entities");
        assert_eq!(symbolic_field_definition.get_container_type(), ContainerType::ArrayFixed(1024));
        assert_eq!(
            symbolic_field_definition.get_display_count_resolution(),
            &SymbolicFieldCountResolution::new_resolver(String::from("entity.count"))
        );
        assert_eq!(symbolic_field_definition.to_string(), "entities:u64[1024] display resolver(entity.count)");
    }

    #[test]
    fn parse_hidden_field_round_trips() {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str("reserved:u8[12] hidden").expect("Expected hidden field definition to parse.");

        assert_eq!(symbolic_field_definition.get_field_name(), "reserved");
        assert_eq!(symbolic_field_definition.get_container_type(), ContainerType::ArrayFixed(12));
        assert!(symbolic_field_definition.is_hidden());
        assert_eq!(symbolic_field_definition.to_string(), "reserved:u8[12] hidden");
    }

    #[test]
    fn parse_active_when_resolver_round_trips() {
        let symbolic_field_definition =
            SymbolicFieldDefinition::from_str("alive:actor.state.alive active resolver(actor.is_alive)").expect("Expected active resolver to parse.");

        assert_eq!(
            symbolic_field_definition
                .get_active_when_resolver()
                .map(|resolver_ref| resolver_ref.get_resolver_id()),
            Some("actor.is_alive")
        );
        assert_eq!(symbolic_field_definition.to_string(), "alive:actor.state.alive active resolver(actor.is_alive)");
    }

    #[test]
    fn parse_dynamic_array_expression_rejects_inline_syntax() {
        let parse_error = SymbolicFieldDefinition::from_str("elements:game.Item[count +] @ +8").expect_err("Expected dynamic array field definition to fail.");

        assert!(parse_error.contains("Dynamic array count must use resolver"));
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
    fn serialized_field_without_name_deserializes_as_anonymous_field() {
        let serialized_value = json!({
            "data_type_ref": { "data_type_id": "u32" },
            "container_type": "None"
        });
        let symbolic_field_definition: SymbolicFieldDefinition =
            serde_json::from_value(serialized_value).expect("Expected symbolic field definition to deserialize.");

        assert_eq!(symbolic_field_definition.get_field_name(), "");
        assert_eq!(symbolic_field_definition.get_data_type_ref(), &DataTypeRef::new("u32"));
        assert_eq!(symbolic_field_definition.get_container_type(), ContainerType::None);
        assert_eq!(symbolic_field_definition.get_count_resolution(), &SymbolicFieldCountResolution::Inferred);
        assert_eq!(symbolic_field_definition.get_offset_resolution(), &SymbolicFieldOffsetResolution::Sequential);
    }
}
