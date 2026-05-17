use crate::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::{
        symbolic_field_definition::{SymbolicFieldCountResolution, SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_resolver_definition::SymbolicResolverRef,
        symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
    },
};
use std::collections::{BTreeSet, HashSet};

pub trait SymbolLayoutDescriptorBuildTarget {
    type Field: SymbolLayoutDescriptorFieldBuildTarget;

    fn get_original_layout_id(&self) -> Option<&str>;
    fn get_layout_id(&self) -> &str;
    fn get_layout_kind(&self) -> SymbolicLayoutKind;
    fn get_size_text(&self) -> &str;
    fn get_size_format(&self) -> AnonymousValueStringFormat;
    fn get_field_count(&self) -> usize;
    fn get_field(
        &self,
        field_position: usize,
    ) -> Option<&Self::Field>;
}

pub trait SymbolLayoutDescriptorFieldBuildTarget {
    fn get_field_name(&self) -> &str;
    fn get_data_type_id(&self) -> &str;
    fn to_container_type(&self) -> Result<ContainerType, String>;
    fn to_count_resolution(&self) -> Result<SymbolicFieldCountResolution, String>;
    fn to_display_count_resolution(&self) -> Result<SymbolicFieldCountResolution, String>;
    fn to_offset_resolution(&self) -> Result<SymbolicFieldOffsetResolution, String>;
    fn to_active_when_resolver(&self) -> Option<SymbolicResolverRef>;
}

pub struct SymbolLayoutDescriptorBuilder;

impl SymbolLayoutDescriptorBuilder {
    pub fn build_symbol_layout_descriptor<Draft>(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &Draft,
    ) -> Result<StructLayoutDescriptor, String>
    where
        Draft: SymbolLayoutDescriptorBuildTarget,
    {
        Self::build_symbol_layout_descriptor_with_unassigned_split_offsets(project_symbol_catalog, draft, &BTreeSet::new())
    }

    pub fn build_symbol_layout_descriptor_with_unassigned_split_offsets<Draft>(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &Draft,
        unassigned_split_offsets: &BTreeSet<u64>,
    ) -> Result<StructLayoutDescriptor, String>
    where
        Draft: SymbolLayoutDescriptorBuildTarget,
    {
        let trimmed_layout_id = draft.get_layout_id().trim();
        if trimmed_layout_id.is_empty() {
            return Err(String::from("Symbol layout id is required."));
        }

        let conflicts_with_existing_layout = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .any(|struct_layout_descriptor| {
                struct_layout_descriptor.get_struct_layout_id() == trimmed_layout_id && draft.get_original_layout_id() != Some(trimmed_layout_id)
            });
        if conflicts_with_existing_layout {
            return Err(String::from("Symbol layout id must be unique."));
        }
        let declared_size_in_bytes = Self::parse_layout_size_text(draft.get_size_text(), draft.get_size_format())?;

        let mut symbolic_field_definitions = Vec::with_capacity(draft.get_field_count());
        let mut field_names = HashSet::new();
        let mut next_sequential_offset = 0_u64;
        for field_position in 0..draft.get_field_count() {
            let Some(field_draft) = draft.get_field(field_position) else {
                continue;
            };
            let trimmed_data_type_id = field_draft.get_data_type_id().trim().to_string();
            if trimmed_data_type_id.is_empty() {
                return Err(String::from("Each field needs a data type."));
            }

            let container_type = field_draft.to_container_type()?;
            let count_resolution = field_draft.to_count_resolution()?;
            let display_count_resolution = field_draft.to_display_count_resolution()?;
            let offset_resolution = field_draft.to_offset_resolution()?;
            let trimmed_field_name = field_draft.get_field_name().trim().to_string();
            if trimmed_field_name.is_empty() {
                return Err(String::from("Each field needs a name."));
            }
            if !field_names.insert(trimmed_field_name.clone()) {
                return Err(format!("Field name `{}` is already used in this layout.", trimmed_field_name));
            }

            let data_type_ref = DataTypeRef::new(&trimmed_data_type_id);
            let symbolic_field_definition = SymbolicFieldDefinition::new_named_with_resolutions_and_display_count(
                trimmed_field_name,
                data_type_ref,
                container_type,
                count_resolution,
                display_count_resolution,
                offset_resolution,
            )
            .with_active_when_resolver(field_draft.to_active_when_resolver());

            let (field_offset, symbolic_field_definition) = match symbolic_field_definition.get_offset_resolution() {
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) if !draft.get_layout_kind().is_union() && *offset_in_bytes >= next_sequential_offset => {
                    if *offset_in_bytes > next_sequential_offset {
                        Self::push_unassigned_range(
                            &mut symbolic_field_definitions,
                            next_sequential_offset,
                            *offset_in_bytes,
                            unassigned_split_offsets,
                            true,
                        );
                    }

                    (
                        *offset_in_bytes,
                        symbolic_field_definition.with_offset_resolution(SymbolicFieldOffsetResolution::Sequential),
                    )
                }
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) => (*offset_in_bytes, symbolic_field_definition),
                SymbolicFieldOffsetResolution::Sequential if draft.get_layout_kind().is_union() => (0, symbolic_field_definition),
                SymbolicFieldOffsetResolution::Sequential => (next_sequential_offset, symbolic_field_definition),
                SymbolicFieldOffsetResolution::Resolver(_) if draft.get_layout_kind().is_union() => (0, symbolic_field_definition),
                SymbolicFieldOffsetResolution::Resolver(_) => (next_sequential_offset, symbolic_field_definition),
            };
            let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(project_symbol_catalog, &symbolic_field_definition, &mut HashSet::new());

            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
            symbolic_field_definitions.push(symbolic_field_definition);
        }
        if !draft.get_layout_kind().is_union() && declared_size_in_bytes > next_sequential_offset {
            Self::push_unassigned_range(
                &mut symbolic_field_definitions,
                next_sequential_offset,
                declared_size_in_bytes,
                unassigned_split_offsets,
                draft.get_field_count() > 0 || !unassigned_split_offsets.is_empty(),
            );
        }

        let symbolic_struct_definition =
            SymbolicStructDefinition::new_with_layout_kind(trimmed_layout_id.to_string(), draft.get_layout_kind(), symbolic_field_definitions)
                .with_declared_size_in_bytes(Some(declared_size_in_bytes));
        let minimum_size_in_bytes = Self::resolve_symbolic_struct_field_span_in_bytes(project_symbol_catalog, &symbolic_struct_definition, &mut HashSet::new());
        if declared_size_in_bytes < minimum_size_in_bytes {
            return Err(format!(
                "Layout size {} byte(s) would truncate fields ending at byte {}.",
                declared_size_in_bytes, minimum_size_in_bytes
            ));
        }

        let struct_layout_descriptor = StructLayoutDescriptor::new(trimmed_layout_id.to_string(), symbolic_struct_definition);

        project_symbol_catalog.validate_local_resolver_dependencies_for_struct_layout(&struct_layout_descriptor)?;

        Ok(struct_layout_descriptor)
    }

    pub fn parse_layout_size_text(
        size_text: &str,
        size_format: AnonymousValueStringFormat,
    ) -> Result<u64, String> {
        let trimmed_size_text = size_text.trim();
        if trimmed_size_text.is_empty() {
            return Err(String::from("Layout size is required."));
        }

        let normalized_size_text = trimmed_size_text
            .strip_prefix('+')
            .map(str::trim)
            .unwrap_or(trimmed_size_text);
        let parsed_size = if let Some(binary_size_text) = normalized_size_text
            .strip_prefix("0b")
            .or_else(|| normalized_size_text.strip_prefix("0B"))
        {
            u64::from_str_radix(binary_size_text, 2)
        } else if let Some(hex_size_text) = normalized_size_text
            .strip_prefix("0x")
            .or_else(|| normalized_size_text.strip_prefix("0X"))
        {
            u64::from_str_radix(hex_size_text, 16)
        } else {
            match size_format {
                AnonymousValueStringFormat::Binary => u64::from_str_radix(normalized_size_text, 2),
                AnonymousValueStringFormat::Decimal => normalized_size_text.parse::<u64>(),
                AnonymousValueStringFormat::Hexadecimal => u64::from_str_radix(normalized_size_text, 16),
                _ => {
                    return Err(format!("Invalid layout size: {}.", trimmed_size_text));
                }
            }
        };

        parsed_size.map_err(|_| format!("Invalid layout size: {}.", trimmed_size_text))
    }

    pub fn resolve_symbolic_struct_size_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_struct_definition: &SymbolicStructDefinition,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> u64 {
        Self::resolve_symbolic_struct_field_span_in_bytes(project_symbol_catalog, symbolic_struct_definition, visited_struct_layout_ids).max(
            symbolic_struct_definition
                .get_declared_size_in_bytes()
                .unwrap_or(0),
        )
    }

    pub fn resolve_symbolic_struct_field_span_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_struct_definition: &SymbolicStructDefinition,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> u64 {
        let mut next_sequential_offset = 0_u64;

        for symbolic_field_definition in symbolic_struct_definition.get_fields() {
            if symbolic_field_definition.is_unassigned() {
                next_sequential_offset = next_sequential_offset.saturating_add(
                    symbolic_field_definition
                        .get_unassigned_size_in_bytes()
                        .unwrap_or(0),
                );
                continue;
            }

            let field_offset = match symbolic_field_definition.get_offset_resolution() {
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                    if symbolic_struct_definition.get_layout_kind().is_union() =>
                {
                    0
                }
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
            };
            let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(project_symbol_catalog, symbolic_field_definition, visited_struct_layout_ids);

            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
        }

        next_sequential_offset
    }

    pub fn resolve_symbolic_field_size_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_field_definition: &SymbolicFieldDefinition,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> u64 {
        if let Some(pointer_size) = symbolic_field_definition
            .get_container_type()
            .get_pointer_size()
        {
            return pointer_size.get_size_in_bytes();
        }

        let data_type_id = symbolic_field_definition.get_data_type_ref().get_data_type_id();
        let unit_size_in_bytes = Self::resolve_data_type_size_in_bytes(project_symbol_catalog, data_type_id, visited_struct_layout_ids);

        symbolic_field_definition
            .get_container_type()
            .get_total_size_in_bytes(unit_size_in_bytes)
    }

    pub fn resolve_primitive_data_type_size_in_bytes(data_type_id: &str) -> Option<u64> {
        match data_type_id {
            "bool" | "i8" | "u8" => Some(1),
            "i16" | "u16" | "i16be" | "u16be" => Some(2),
            "i24" | "u24" | "i24be" | "u24be" => Some(3),
            "f32" | "i32" | "u32" | "f32be" | "i32be" | "u32be" => Some(4),
            "f64" | "i64" | "u64" | "f64be" | "i64be" | "u64be" => Some(8),
            "i128" | "u128" | "i128be" | "u128be" => Some(16),
            _ => None,
        }
    }

    fn push_unassigned_range(
        symbolic_field_definitions: &mut Vec<SymbolicFieldDefinition>,
        start_offset_in_bytes: u64,
        end_offset_in_bytes: u64,
        unassigned_split_offsets: &BTreeSet<u64>,
        include_unsplit_range: bool,
    ) {
        if end_offset_in_bytes <= start_offset_in_bytes {
            return;
        }

        let contained_split_offsets = unassigned_split_offsets
            .range((start_offset_in_bytes.saturating_add(1))..end_offset_in_bytes)
            .copied()
            .collect::<Vec<_>>();
        if contained_split_offsets.is_empty() && !include_unsplit_range {
            return;
        }

        let mut previous_offset_in_bytes = start_offset_in_bytes;
        for split_offset_in_bytes in contained_split_offsets {
            if split_offset_in_bytes > previous_offset_in_bytes {
                symbolic_field_definitions.push(SymbolicFieldDefinition::new_unassigned(
                    split_offset_in_bytes.saturating_sub(previous_offset_in_bytes),
                ));
            }
            previous_offset_in_bytes = split_offset_in_bytes;
        }

        if end_offset_in_bytes > previous_offset_in_bytes {
            symbolic_field_definitions.push(SymbolicFieldDefinition::new_unassigned(
                end_offset_in_bytes.saturating_sub(previous_offset_in_bytes),
            ));
        }
    }

    fn resolve_data_type_size_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        data_type_id: &str,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> u64 {
        if !visited_struct_layout_ids.insert(data_type_id.to_string()) {
            return 0;
        }

        let size_in_bytes = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == data_type_id)
            .map(|struct_layout_descriptor| {
                Self::resolve_symbolic_struct_size_in_bytes(
                    project_symbol_catalog,
                    struct_layout_descriptor.get_struct_layout_definition(),
                    visited_struct_layout_ids,
                )
            })
            .or_else(|| Self::resolve_primitive_data_type_size_in_bytes(data_type_id))
            .unwrap_or(1);

        visited_struct_layout_ids.remove(data_type_id);

        size_in_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestDraft {
        original_layout_id: Option<String>,
        layout_id: String,
        layout_kind: SymbolicLayoutKind,
        size_text: String,
        fields: Vec<TestField>,
    }

    #[derive(Clone, Debug)]
    struct TestField {
        field_name: String,
        data_type_id: String,
        offset_resolution: SymbolicFieldOffsetResolution,
    }

    impl SymbolLayoutDescriptorBuildTarget for TestDraft {
        type Field = TestField;

        fn get_original_layout_id(&self) -> Option<&str> {
            self.original_layout_id.as_deref()
        }

        fn get_layout_id(&self) -> &str {
            &self.layout_id
        }

        fn get_layout_kind(&self) -> SymbolicLayoutKind {
            self.layout_kind
        }

        fn get_size_text(&self) -> &str {
            &self.size_text
        }

        fn get_size_format(&self) -> AnonymousValueStringFormat {
            AnonymousValueStringFormat::Decimal
        }

        fn get_field_count(&self) -> usize {
            self.fields.len()
        }

        fn get_field(
            &self,
            field_position: usize,
        ) -> Option<&Self::Field> {
            self.fields.get(field_position)
        }
    }

    impl SymbolLayoutDescriptorFieldBuildTarget for TestField {
        fn get_field_name(&self) -> &str {
            &self.field_name
        }

        fn get_data_type_id(&self) -> &str {
            &self.data_type_id
        }

        fn to_container_type(&self) -> Result<ContainerType, String> {
            Ok(ContainerType::None)
        }

        fn to_count_resolution(&self) -> Result<SymbolicFieldCountResolution, String> {
            Ok(SymbolicFieldCountResolution::Inferred)
        }

        fn to_display_count_resolution(&self) -> Result<SymbolicFieldCountResolution, String> {
            Ok(SymbolicFieldCountResolution::Inferred)
        }

        fn to_offset_resolution(&self) -> Result<SymbolicFieldOffsetResolution, String> {
            Ok(self.offset_resolution.clone())
        }

        fn to_active_when_resolver(&self) -> Option<SymbolicResolverRef> {
            None
        }
    }

    #[test]
    fn descriptor_builder_materializes_static_gap_as_unassigned() {
        let draft = TestDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("12"),
            fields: vec![TestField {
                field_name: String::from("health"),
                data_type_id: String::from("u32"),
                offset_resolution: SymbolicFieldOffsetResolution::Static(4),
            }],
        };

        let descriptor =
            SymbolLayoutDescriptorBuilder::build_symbol_layout_descriptor(&ProjectSymbolCatalog::default(), &draft).expect("Expected descriptor to build.");
        let fields = descriptor.get_struct_layout_definition().get_fields();

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].to_string(), "unassigned[4]");
        assert_eq!(fields[1].to_string(), "health:u32");
        assert_eq!(fields[2].to_string(), "unassigned[4]");
    }
}
