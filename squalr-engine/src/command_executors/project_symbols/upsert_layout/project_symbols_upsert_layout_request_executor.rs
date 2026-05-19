use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::{
    project_symbol_catalog_persistence::save_and_sync_project_symbol_catalog, project_symbol_layout_mutation::ProjectSymbolLayoutMutation,
};
use squalr_engine_api::commands::project_symbols::upsert_layout::project_symbols_upsert_layout_request::ProjectSymbolsUpsertLayoutRequest;
use squalr_engine_api::commands::project_symbols::upsert_layout::project_symbols_upsert_layout_response::ProjectSymbolsUpsertLayoutResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsUpsertLayoutRequest {
    type ResponseType = ProjectSymbolsUpsertLayoutResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let struct_layout_descriptor = match self.to_struct_layout_descriptor() {
            Ok(struct_layout_descriptor) => struct_layout_descriptor,
            Err(error) => {
                return ProjectSymbolsUpsertLayoutResponse {
                    success: false,
                    struct_layout_id: self.struct_layout_id.clone(),
                    error: Some(error),
                };
            }
        };
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                return ProjectSymbolsUpsertLayoutResponse {
                    success: false,
                    struct_layout_id: self.struct_layout_id.clone(),
                    error: Some(format!(
                        "Failed to acquire opened project lock for project-symbols upsert-layout command: {error}"
                    )),
                };
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            return ProjectSymbolsUpsertLayoutResponse {
                success: false,
                struct_layout_id: self.struct_layout_id.clone(),
                error: Some(String::from("Cannot upsert symbol layout without an opened project.")),
            };
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            return ProjectSymbolsUpsertLayoutResponse {
                success: false,
                struct_layout_id: self.struct_layout_id.clone(),
                error: Some(String::from(
                    "Failed to resolve opened project directory for project-symbols upsert-layout command.",
                )),
            };
        };

        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();
        if let Err(error) = ProjectSymbolLayoutMutation::upsert_struct_layout_descriptor(
            project_symbol_catalog,
            self.original_struct_layout_id.as_deref(),
            struct_layout_descriptor,
        ) {
            return ProjectSymbolsUpsertLayoutResponse {
                success: false,
                struct_layout_id: self.struct_layout_id.clone(),
                error: Some(error),
            };
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsUpsertLayoutResponse {
                success: false,
                struct_layout_id: self.struct_layout_id.clone(),
                error: Some(String::from("Failed to save and sync project symbol catalog.")),
            };
        }

        ProjectSymbolsUpsertLayoutResponse {
            success: true,
            struct_layout_id: self.struct_layout_id.clone(),
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsUpsertLayoutRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_values::container_type::ContainerType;
    use squalr_engine_api::structures::projects::{project::Project, project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim};
    use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn upsert_layout_request_persists_layout_and_syncs_catalog() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project = create_project_with_symbol_catalog(temp_directory.path(), ProjectSymbolCatalog::default());
        let mock_project_symbols_bindings = MockProjectSymbolsBindings::new();
        let captured_project_symbol_catalogs = mock_project_symbols_bindings.captured_project_symbol_catalogs();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_project_symbols_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let response = ProjectSymbolsUpsertLayoutRequest {
            original_struct_layout_id: None,
            struct_layout_id: String::from("player.stats"),
            layout_kind: String::from("struct"),
            size_in_bytes: Some(8),
            field_definitions: vec![String::from("health:u32"), String::from("ammo:u32")],
        }
        .execute(&engine_execution_context);

        assert!(response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected layout project to load from disk.");
        let struct_layout_descriptors = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_struct_layout_descriptors();

        assert_eq!(struct_layout_descriptors.len(), 1);
        assert_eq!(struct_layout_descriptors[0].get_struct_layout_id(), "player.stats");
        assert_eq!(
            struct_layout_descriptors[0]
                .get_struct_layout_definition()
                .get_fields()
                .len(),
            2
        );

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(
            captured_project_symbol_catalogs[0].get_struct_layout_descriptors()[0].get_struct_layout_id(),
            "player.stats"
        );
    }

    #[test]
    fn upsert_layout_request_retargets_references_when_renamed() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("player.old"),
                SymbolicStructDefinition::new(
                    String::from("player.old"),
                    vec![SymbolicFieldDefinition::new_named(
                        String::from("health"),
                        DataTypeRef::new("u32"),
                        ContainerType::None,
                    )],
                ),
            )],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player.old"),
            )],
        );
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let response = ProjectSymbolsUpsertLayoutRequest {
            original_struct_layout_id: Some(String::from("player.old")),
            struct_layout_id: String::from("player.new"),
            layout_kind: String::from("struct"),
            size_in_bytes: None,
            field_definitions: vec![String::from("health:u32")],
        }
        .execute(&engine_execution_context);

        assert!(response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected renamed-layout project to load from disk.");
        let project_symbol_catalog = loaded_project.get_project_info().get_project_symbol_catalog();

        assert_eq!(project_symbol_catalog.get_struct_layout_descriptors()[0].get_struct_layout_id(), "player.new");
        assert_eq!(project_symbol_catalog.get_symbol_claims()[0].get_struct_layout_id(), "player.new");
    }
}
