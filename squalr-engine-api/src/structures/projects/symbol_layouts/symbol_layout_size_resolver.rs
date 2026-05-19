use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    structs::{
        symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_struct_definition::SymbolicStructDefinition,
    },
};
use std::collections::HashSet;

pub struct SymbolLayoutSizeResolver;

impl SymbolLayoutSizeResolver {
    pub fn resolve_symbolic_field_size_in_bytes<ResolveDataTypeSize, ResolveStructLayout>(
        symbolic_field_definition: &SymbolicFieldDefinition,
        resolve_data_type_size_in_bytes: ResolveDataTypeSize,
        resolve_struct_layout_definition: ResolveStructLayout,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> Option<u64>
    where
        ResolveDataTypeSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
        ResolveStructLayout: Fn(&str) -> Option<SymbolicStructDefinition> + Copy,
    {
        if symbolic_field_definition.is_unassigned() {
            return symbolic_field_definition.get_unassigned_size_in_bytes();
        }
        if let Some(pointer_size) = symbolic_field_definition
            .get_container_type()
            .get_pointer_size()
        {
            return Some(pointer_size.get_size_in_bytes());
        }

        let data_type_ref = symbolic_field_definition.get_data_type_ref();
        let unit_size_in_bytes = if let Some(data_type_size_in_bytes) = resolve_data_type_size_in_bytes(data_type_ref) {
            data_type_size_in_bytes
        } else {
            let struct_layout_id = data_type_ref.get_data_type_id();

            if !visited_struct_layout_ids.insert(struct_layout_id.to_string()) {
                return None;
            }

            let symbolic_struct_definition = resolve_struct_layout_definition(struct_layout_id)?;
            let struct_size_in_bytes = Self::resolve_symbolic_struct_size_in_bytes(
                &symbolic_struct_definition,
                resolve_data_type_size_in_bytes,
                resolve_struct_layout_definition,
                visited_struct_layout_ids,
            )?;

            visited_struct_layout_ids.remove(struct_layout_id);
            struct_size_in_bytes
        };

        Some(
            symbolic_field_definition
                .get_container_type()
                .get_total_size_in_bytes(unit_size_in_bytes),
        )
    }

    pub fn resolve_symbolic_struct_size_in_bytes<ResolveDataTypeSize, ResolveStructLayout>(
        symbolic_struct_definition: &SymbolicStructDefinition,
        resolve_data_type_size_in_bytes: ResolveDataTypeSize,
        resolve_struct_layout_definition: ResolveStructLayout,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> Option<u64>
    where
        ResolveDataTypeSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
        ResolveStructLayout: Fn(&str) -> Option<SymbolicStructDefinition> + Copy,
    {
        let field_span_in_bytes = Self::resolve_symbolic_struct_field_span_in_bytes(
            symbolic_struct_definition,
            resolve_data_type_size_in_bytes,
            resolve_struct_layout_definition,
            visited_struct_layout_ids,
        )?;

        Some(
            field_span_in_bytes.max(
                symbolic_struct_definition
                    .get_declared_size_in_bytes()
                    .unwrap_or(0),
            ),
        )
    }

    pub fn resolve_symbolic_struct_field_span_in_bytes<ResolveDataTypeSize, ResolveStructLayout>(
        symbolic_struct_definition: &SymbolicStructDefinition,
        resolve_data_type_size_in_bytes: ResolveDataTypeSize,
        resolve_struct_layout_definition: ResolveStructLayout,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> Option<u64>
    where
        ResolveDataTypeSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
        ResolveStructLayout: Fn(&str) -> Option<SymbolicStructDefinition> + Copy,
    {
        let mut next_sequential_offset = 0_u64;

        for symbolic_field_definition in symbolic_struct_definition.get_fields() {
            if symbolic_field_definition.is_unassigned() {
                next_sequential_offset = next_sequential_offset.saturating_add(symbolic_field_definition.get_unassigned_size_in_bytes()?);
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
            let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(
                symbolic_field_definition,
                resolve_data_type_size_in_bytes,
                resolve_struct_layout_definition,
                visited_struct_layout_ids,
            )?;
            let field_end_offset = field_offset.checked_add(field_size_in_bytes)?;

            next_sequential_offset = next_sequential_offset.max(field_end_offset);
        }

        Some(next_sequential_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        structs::{
            symbolic_field_definition::SymbolicFieldDefinition,
            symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
        },
    };

    #[test]
    fn struct_size_includes_explicit_unassigned_entries() {
        let symbolic_struct_definition = SymbolicStructDefinition::new_with_layout_kind(
            String::from("player"),
            SymbolicLayoutKind::Struct,
            vec![
                SymbolicFieldDefinition::new_unassigned(4),
                SymbolicFieldDefinition::new_named(String::from("health"), DataTypeRef::new("u32"), ContainerType::None),
                SymbolicFieldDefinition::new_unassigned(4),
            ],
        );

        let size_in_bytes = SymbolLayoutSizeResolver::resolve_symbolic_struct_size_in_bytes(
            &symbolic_struct_definition,
            |data_type_ref| (data_type_ref.get_data_type_id() == "u32").then_some(4),
            |_struct_layout_id| None,
            &mut HashSet::new(),
        );

        assert_eq!(size_in_bytes, Some(12));
    }
}
