use crate::structures::structs::{
    symbol_resolver::SymbolResolver,
    symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
    symbolic_struct_ref::SymbolicStructRef,
    valued_struct::ValuedStruct,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolicLayoutKind {
    #[default]
    Struct,
    Union,
}

impl SymbolicLayoutKind {
    pub const ALL: [Self; 2] = [Self::Struct, Self::Union];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Struct => "Struct",
            Self::Union => "Union",
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Union => "union",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        let trimmed_key = key.trim();

        Self::ALL
            .iter()
            .copied()
            .find(|layout_kind| layout_kind.key() == trimmed_key)
    }

    pub fn is_union(&self) -> bool {
        matches!(self, Self::Union)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolicStructDefinition {
    symbol_namespace: String,
    #[serde(default)]
    layout_kind: SymbolicLayoutKind,
    size_in_bytes: Option<u64>,
    fields: Vec<SymbolicFieldDefinition>,
}

impl SymbolicStructDefinition {
    pub fn new(
        symbol_namespace: String,
        fields: Vec<SymbolicFieldDefinition>,
    ) -> Self {
        SymbolicStructDefinition {
            symbol_namespace,
            layout_kind: SymbolicLayoutKind::Struct,
            size_in_bytes: None,
            fields,
        }
    }

    pub fn new_with_layout_kind(
        symbol_namespace: String,
        layout_kind: SymbolicLayoutKind,
        fields: Vec<SymbolicFieldDefinition>,
    ) -> Self {
        SymbolicStructDefinition {
            symbol_namespace,
            layout_kind,
            size_in_bytes: None,
            fields,
        }
    }

    pub fn new_union(
        symbol_namespace: String,
        fields: Vec<SymbolicFieldDefinition>,
    ) -> Self {
        Self::new_with_layout_kind(symbol_namespace, SymbolicLayoutKind::Union, fields)
    }

    pub fn new_anonymous(fields: Vec<SymbolicFieldDefinition>) -> Self {
        SymbolicStructDefinition {
            symbol_namespace: String::new(),
            layout_kind: SymbolicLayoutKind::Struct,
            size_in_bytes: None,
            fields,
        }
    }

    pub fn new_anonymous_with_layout_kind(
        layout_kind: SymbolicLayoutKind,
        fields: Vec<SymbolicFieldDefinition>,
    ) -> Self {
        SymbolicStructDefinition {
            symbol_namespace: String::new(),
            layout_kind,
            size_in_bytes: None,
            fields,
        }
    }

    pub fn get_symbol_namespace(&self) -> &str {
        &self.symbol_namespace
    }

    pub fn get_layout_kind(&self) -> SymbolicLayoutKind {
        self.layout_kind
    }

    pub fn get_declared_size_in_bytes(&self) -> Option<u64> {
        self.size_in_bytes
    }

    pub fn get_fields(&self) -> &[SymbolicFieldDefinition] {
        &self.fields
    }

    pub fn with_declared_size_in_bytes(
        mut self,
        size_in_bytes: Option<u64>,
    ) -> Self {
        self.size_in_bytes = size_in_bytes;
        self
    }

    pub fn set_declared_size_in_bytes(
        &mut self,
        size_in_bytes: Option<u64>,
    ) {
        self.size_in_bytes = size_in_bytes;
    }

    pub fn add_field(
        &mut self,
        symbolic_struct_field: SymbolicFieldDefinition,
    ) {
        self.fields.push(symbolic_struct_field);
    }

    pub fn get_default_valued_struct(
        &self,
        symbol_registry: &impl SymbolResolver,
    ) -> ValuedStruct {
        let fields = self
            .fields
            .iter()
            .filter(|field| !field.is_unassigned())
            .map(|field| field.get_valued_struct_field(symbol_registry, false))
            .collect();
        ValuedStruct::new_with_layout_kind(SymbolicStructRef::new(self.symbol_namespace.clone()), self.layout_kind, fields)
    }

    pub fn get_size_in_bytes(
        &self,
        symbol_registry: &impl SymbolResolver,
    ) -> u64 {
        let mut next_sequential_offset = 0_u64;

        for field in &self.fields {
            let field_offset = match field.get_offset_resolution() {
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) if self.layout_kind.is_union() => 0,
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
            };
            let field_size_in_bytes = field.get_size_in_bytes(symbol_registry);

            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
        }

        next_sequential_offset.max(self.size_in_bytes.unwrap_or(0))
    }
}

impl FromStr for SymbolicStructDefinition {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let fields: Result<Vec<SymbolicFieldDefinition>, Self::Err> = string
            .split(';')
            .filter(|&field_string| !field_string.is_empty())
            .map(|field_string| SymbolicFieldDefinition::from_str(field_string))
            .collect();

        Ok(SymbolicStructDefinition::new(String::new(), fields?))
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolicStructDefinition;
    use crate::registries::symbols::symbol_registry::SymbolRegistry;
    use crate::structures::structs::symbolic_field_definition::SymbolicFieldDefinition;
    use crate::structures::structs::symbolic_struct_definition::SymbolicLayoutKind;
    use std::str::FromStr;

    #[test]
    fn deserializes_legacy_struct_definition_without_layout_kind() {
        let symbolic_struct_definition: SymbolicStructDefinition =
            serde_json::from_str(r#"{"symbol_namespace":"legacy.layout","fields":[]}"#).expect("Expected legacy struct definition to deserialize.");

        assert_eq!(symbolic_struct_definition.get_symbol_namespace(), "legacy.layout");
        assert_eq!(symbolic_struct_definition.get_layout_kind(), SymbolicLayoutKind::Struct);
    }

    #[test]
    fn get_size_in_bytes_uses_static_field_span_for_overlapping_layouts() {
        let symbol_registry = SymbolRegistry::new();
        let symbolic_struct_definition = SymbolicStructDefinition::from_str("wide:u64 @ +0;narrow:u32 @ +0;tail:u16 @ +8")
            .expect("Expected union-like symbolic struct definition to parse.");

        assert_eq!(symbolic_struct_definition.get_size_in_bytes(&symbol_registry), 10);
    }

    #[test]
    fn union_layout_size_defaults_fields_to_same_offset() {
        let symbol_registry = SymbolRegistry::new();
        let symbolic_struct_definition = SymbolicStructDefinition::new_union(
            String::from("Variant"),
            vec![
                SymbolicFieldDefinition::from_str("as_u32:u32").expect("Expected u32 union field to parse."),
                SymbolicFieldDefinition::from_str("raw:u8[16]").expect("Expected raw union field to parse."),
            ],
        );

        assert_eq!(symbolic_struct_definition.get_size_in_bytes(&symbol_registry), 16);
    }

    #[test]
    fn declared_size_extends_layout_past_field_span() {
        let symbol_registry = SymbolRegistry::new();
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Sized"),
            vec![SymbolicFieldDefinition::from_str("value:u32").expect("Expected field to parse.")],
        )
        .with_declared_size_in_bytes(Some(32));

        assert_eq!(symbolic_struct_definition.get_size_in_bytes(&symbol_registry), 32);
        assert_eq!(symbolic_struct_definition.get_declared_size_in_bytes(), Some(32));
    }

    #[test]
    fn unassigned_entries_advance_sequential_layout_size() {
        let symbol_registry = SymbolRegistry::new();
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Sparse"),
            vec![
                SymbolicFieldDefinition::from_str("unassigned[0x20]").expect("Expected unassigned entry to parse."),
                SymbolicFieldDefinition::from_str("value:u32").expect("Expected value field to parse."),
            ],
        );

        assert_eq!(symbolic_struct_definition.get_size_in_bytes(&symbol_registry), 0x24);
    }
}
