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
