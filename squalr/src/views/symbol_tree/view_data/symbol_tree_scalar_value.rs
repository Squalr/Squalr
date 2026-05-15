use squalr_engine_api::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::container_type::ContainerType,
    projects::project_symbol_locator::ProjectSymbolLocator,
    structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
};
use squalr_engine_session::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;
use std::collections::HashSet;

pub struct SymbolTreeScalarValue {}

impl SymbolTreeScalarValue {
    pub fn build_query(
        project_symbol_locator: &ProjectSymbolLocator,
        field_definition: &SymbolicFieldDefinition,
        field_size_in_bytes: u64,
        supports_scalar_integer_values: impl Fn(&DataTypeRef) -> bool,
    ) -> Option<VirtualSnapshotQuery> {
        if field_size_in_bytes == 0 || !supports_scalar_integer_values(field_definition.get_data_type_ref()) {
            return None;
        }

        let scalar_field_definition = SymbolicFieldDefinition::new_named(
            field_definition.get_field_name().to_string(),
            field_definition.get_data_type_ref().clone(),
            ContainerType::None,
        );
        let symbolic_struct_definition = SymbolicStructDefinition::new_anonymous(vec![scalar_field_definition]);

        Some(VirtualSnapshotQuery::Address {
            query_id: Self::query_id(project_symbol_locator, field_definition),
            address: project_symbol_locator.get_focus_address(),
            module_name: project_symbol_locator.get_focus_module_name().to_string(),
            symbolic_struct_definition,
        })
    }

    pub fn query_id(
        project_symbol_locator: &ProjectSymbolLocator,
        field_definition: &SymbolicFieldDefinition,
    ) -> String {
        format!(
            "scalar:{}:{}:{}",
            project_symbol_locator.to_locator_key(),
            field_definition.get_field_name(),
            field_definition.get_data_type_ref()
        )
    }

    pub fn deduplicate_queries_by_id(virtual_snapshot_queries: Vec<VirtualSnapshotQuery>) -> Vec<VirtualSnapshotQuery> {
        let mut seen_query_ids = HashSet::new();
        let mut deduplicated_virtual_snapshot_queries = Vec::new();

        for virtual_snapshot_query in virtual_snapshot_queries {
            if seen_query_ids.insert(virtual_snapshot_query.get_query_id().to_string()) {
                deduplicated_virtual_snapshot_queries.push(virtual_snapshot_query);
            }
        }

        deduplicated_virtual_snapshot_queries
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolTreeScalarValue;
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef, data_values::container_type::ContainerType, projects::project_symbol_locator::ProjectSymbolLocator,
        structs::symbolic_field_definition::SymbolicFieldDefinition,
    };
    use squalr_engine_session::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;

    #[test]
    fn build_query_reads_scalar_at_resolved_locator() {
        let field_definition = SymbolicFieldDefinition::new_named(String::from("e_lfanew"), DataTypeRef::new("u32"), ContainerType::None);
        let project_symbol_locator = ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x3C);
        let scalar_query = SymbolTreeScalarValue::build_query(&project_symbol_locator, &field_definition, 4, |data_type_ref| {
            data_type_ref.get_data_type_id() == "u32"
        })
        .expect("Expected scalar query.");

        let (query_id, address, module_name, symbolic_struct_definition) = match scalar_query {
            VirtualSnapshotQuery::Address {
                query_id,
                address,
                module_name,
                symbolic_struct_definition,
            } => Some((query_id, address, module_name, symbolic_struct_definition)),
            VirtualSnapshotQuery::Pointer { .. } => None,
        }
        .expect("Expected address scalar query.");

        assert_eq!(query_id, "scalar:module:game.exe:3C:e_lfanew:u32");
        assert_eq!(address, 0x3C);
        assert_eq!(module_name, "game.exe");
        assert_eq!(symbolic_struct_definition.get_fields().len(), 1);
        assert_eq!(symbolic_struct_definition.get_fields()[0].get_field_name(), "e_lfanew");
        assert_eq!(symbolic_struct_definition.get_fields()[0].get_data_type_ref(), &DataTypeRef::new("u32"));
    }

    #[test]
    fn build_query_ignores_data_types_without_scalar_integer_support() {
        let field_definition = SymbolicFieldDefinition::new_named(String::from("title"), DataTypeRef::new("string"), ContainerType::None);
        let project_symbol_locator = ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x3C);
        let scalar_query = SymbolTreeScalarValue::build_query(&project_symbol_locator, &field_definition, 1, |_| false);

        assert!(scalar_query.is_none());
    }
}
