use crate::structures::{
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::{
        symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_struct_definition::SymbolicStructDefinition,
    },
};
use std::collections::HashSet;

/// Counts the rows a layout presents when implicit unassigned gaps are materialized.
pub struct SymbolLayoutVisibleEntryCounter;

impl SymbolLayoutVisibleEntryCounter {
    pub fn count_visible_entries(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_struct_definition: &SymbolicStructDefinition,
    ) -> usize {
        if symbolic_struct_definition.get_layout_kind().is_union() {
            return symbolic_struct_definition.get_fields().len();
        }

        let mut visible_entry_count = 0_usize;
        let mut next_sequential_offset = 0_u64;

        for symbolic_field_definition in symbolic_struct_definition.get_fields() {
            if symbolic_field_definition.is_unassigned() {
                if symbolic_field_definition
                    .get_unassigned_size_in_bytes()
                    .unwrap_or(0)
                    > 0
                {
                    visible_entry_count = visible_entry_count.saturating_add(1);
                }
                next_sequential_offset = next_sequential_offset.saturating_add(
                    symbolic_field_definition
                        .get_unassigned_size_in_bytes()
                        .unwrap_or(0),
                );
                continue;
            }

            let field_offset = match symbolic_field_definition.get_offset_resolution() {
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
            };

            if field_offset > next_sequential_offset {
                visible_entry_count = visible_entry_count.saturating_add(1);
            }

            let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(project_symbol_catalog, symbolic_field_definition, &mut HashSet::new());
            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
            visible_entry_count = visible_entry_count.saturating_add(1);
        }

        if symbolic_struct_definition
            .get_declared_size_in_bytes()
            .unwrap_or(0)
            > next_sequential_offset
        {
            visible_entry_count = visible_entry_count.saturating_add(1);
        }

        visible_entry_count
    }

    fn resolve_symbolic_field_size_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_field_definition: &SymbolicFieldDefinition,
        visited_type_ids: &mut HashSet<String>,
    ) -> u64 {
        if symbolic_field_definition.is_unassigned() {
            return symbolic_field_definition
                .get_unassigned_size_in_bytes()
                .unwrap_or(0);
        }
        if let Some(pointer_size) = symbolic_field_definition
            .get_container_type()
            .get_pointer_size()
        {
            return pointer_size.get_size_in_bytes();
        }

        let data_type_id = symbolic_field_definition.get_data_type_ref().get_data_type_id();
        let unit_size_in_bytes = Self::resolve_primitive_data_type_size_in_bytes(data_type_id).unwrap_or_else(|| {
            if !visited_type_ids.insert(data_type_id.to_string()) {
                return 0;
            }

            let struct_size_in_bytes = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == data_type_id)
                .map(|struct_layout_descriptor| {
                    Self::resolve_symbolic_struct_size_in_bytes(
                        project_symbol_catalog,
                        struct_layout_descriptor.get_struct_layout_definition(),
                        visited_type_ids,
                    )
                })
                .unwrap_or(1);

            visited_type_ids.remove(data_type_id);
            struct_size_in_bytes
        });

        symbolic_field_definition
            .get_container_type()
            .get_total_size_in_bytes(unit_size_in_bytes)
    }

    fn resolve_symbolic_struct_size_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_struct_definition: &SymbolicStructDefinition,
        visited_type_ids: &mut HashSet<String>,
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
            let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(project_symbol_catalog, symbolic_field_definition, visited_type_ids);

            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
        }

        next_sequential_offset.max(
            symbolic_struct_definition
                .get_declared_size_in_bytes()
                .unwrap_or(0),
        )
    }

    fn resolve_primitive_data_type_size_in_bytes(data_type_id: &str) -> Option<u64> {
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
}

#[cfg(test)]
mod tests {
    use super::SymbolLayoutVisibleEntryCounter;
    use crate::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        projects::project_symbol_catalog::ProjectSymbolCatalog,
        structs::{
            symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
            symbolic_struct_definition::SymbolicStructDefinition,
        },
    };

    #[test]
    fn counts_implicit_tail_unassigned_entry_after_static_field() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("game.exe"),
            vec![SymbolicFieldDefinition::new_named_with_resolutions(
                String::from("Pointer"),
                DataTypeRef::new("u32"),
                ContainerType::None,
                Default::default(),
                SymbolicFieldOffsetResolution::new_static(0x579C),
            )],
        )
        .with_declared_size_in_bytes(Some(0x10000));

        assert_eq!(
            SymbolLayoutVisibleEntryCounter::count_visible_entries(&ProjectSymbolCatalog::default(), &symbolic_struct_definition),
            3
        );
    }

    #[test]
    fn counts_nested_layout_then_gap_field_and_tail() {
        let child_layout = SymbolicStructDefinition::new(String::from("pe.headers"), Vec::new()).with_declared_size_in_bytes(Some(0x100));
        let module_layout = SymbolicStructDefinition::new(
            String::from("game.exe"),
            vec![
                SymbolicFieldDefinition::new_named_with_resolutions(
                    String::from("PE"),
                    DataTypeRef::new("pe.headers"),
                    ContainerType::None,
                    Default::default(),
                    SymbolicFieldOffsetResolution::new_static(0),
                ),
                SymbolicFieldDefinition::new_named_with_resolutions(
                    String::from("Pointer"),
                    DataTypeRef::new("u32"),
                    ContainerType::None,
                    Default::default(),
                    SymbolicFieldOffsetResolution::new_static(0x579C),
                ),
            ],
        )
        .with_declared_size_in_bytes(Some(0x10000));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("pe.headers"),
                child_layout,
            )],
            Vec::new(),
        );

        assert_eq!(
            SymbolLayoutVisibleEntryCounter::count_visible_entries(&project_symbol_catalog, &module_layout),
            4
        );
    }
}
