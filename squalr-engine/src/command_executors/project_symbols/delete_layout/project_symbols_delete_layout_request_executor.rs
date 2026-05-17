use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::{
    project_symbol_catalog_persistence::save_and_sync_project_symbol_catalog, project_symbol_layout_mutation::ProjectSymbolLayoutMutation,
};
use squalr_engine_api::commands::project_symbols::delete_layout::project_symbols_delete_layout_request::ProjectSymbolsDeleteLayoutRequest;
use squalr_engine_api::commands::project_symbols::delete_layout::project_symbols_delete_layout_response::ProjectSymbolsDeleteLayoutResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsDeleteLayoutRequest {
    type ResponseType = ProjectSymbolsDeleteLayoutResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                return ProjectSymbolsDeleteLayoutResponse {
                    success: false,
                    struct_layout_id: self.struct_layout_id.clone(),
                    error: Some(format!(
                        "Failed to acquire opened project lock for project-symbols delete-layout command: {error}"
                    )),
                };
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            return ProjectSymbolsDeleteLayoutResponse {
                success: false,
                struct_layout_id: self.struct_layout_id.clone(),
                error: Some(String::from("Cannot delete symbol layout without an opened project.")),
            };
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            return ProjectSymbolsDeleteLayoutResponse {
                success: false,
                struct_layout_id: self.struct_layout_id.clone(),
                error: Some(String::from(
                    "Failed to resolve opened project directory for project-symbols delete-layout command.",
                )),
            };
        };

        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();
        if let Err(error) =
            ProjectSymbolLayoutMutation::delete_struct_layout(project_symbol_catalog, &self.struct_layout_id, DataTypeRef::new(&self.replacement_data_type_id))
        {
            return ProjectSymbolsDeleteLayoutResponse {
                success: false,
                struct_layout_id: self.struct_layout_id.clone(),
                error: Some(error),
            };
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsDeleteLayoutResponse {
                success: false,
                struct_layout_id: self.struct_layout_id.clone(),
                error: Some(String::from("Failed to save and sync project symbol catalog.")),
            };
        }

        ProjectSymbolsDeleteLayoutResponse {
            success: true,
            struct_layout_id: self.struct_layout_id.clone(),
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsDeleteLayoutRequest;
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
    fn delete_layout_request_removes_layout_and_retargets_references() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("player.stats"),
                SymbolicStructDefinition::new(
                    String::from("player.stats"),
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
                String::from("player.stats"),
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
        let response = ProjectSymbolsDeleteLayoutRequest::new("player.stats").execute(&engine_execution_context);

        assert!(response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected delete-layout project to load from disk.");
        let project_symbol_catalog = loaded_project.get_project_info().get_project_symbol_catalog();

        assert!(
            project_symbol_catalog
                .get_struct_layout_descriptors()
                .is_empty()
        );
        assert_eq!(project_symbol_catalog.get_symbol_claims()[0].get_struct_layout_id(), "u8");
    }
}
