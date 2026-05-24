use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::{
    project_symbol_catalog_persistence::save_and_sync_project_symbol_catalog, project_symbol_resolver_mutation::ProjectSymbolResolverMutation,
};
use squalr_engine_api::commands::project_symbols::upsert_resolver::project_symbols_upsert_resolver_request::ProjectSymbolsUpsertResolverRequest;
use squalr_engine_api::commands::project_symbols::upsert_resolver::project_symbols_upsert_resolver_response::ProjectSymbolsUpsertResolverResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsUpsertResolverRequest {
    type ResponseType = ProjectSymbolsUpsertResolverResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let resolver_descriptor = match self.to_resolver_descriptor() {
            Ok(resolver_descriptor) => resolver_descriptor,
            Err(error) => {
                return ProjectSymbolsUpsertResolverResponse {
                    success: false,
                    resolver_id: self.resolver_id.clone(),
                    error: Some(error),
                };
            }
        };
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                return ProjectSymbolsUpsertResolverResponse {
                    success: false,
                    resolver_id: self.resolver_id.clone(),
                    error: Some(format!(
                        "Failed to acquire opened project lock for project-symbols upsert-resolver command: {error}"
                    )),
                };
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            return ProjectSymbolsUpsertResolverResponse {
                success: false,
                resolver_id: self.resolver_id.clone(),
                error: Some(String::from("Cannot upsert symbol resolver without an opened project.")),
            };
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            return ProjectSymbolsUpsertResolverResponse {
                success: false,
                resolver_id: self.resolver_id.clone(),
                error: Some(String::from(
                    "Failed to resolve opened project directory for project-symbols upsert-resolver command.",
                )),
            };
        };

        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();
        if let Err(error) =
            ProjectSymbolResolverMutation::upsert_resolver_descriptor(project_symbol_catalog, self.original_resolver_id.as_deref(), resolver_descriptor)
        {
            return ProjectSymbolsUpsertResolverResponse {
                success: false,
                resolver_id: self.resolver_id.clone(),
                error: Some(error),
            };
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsUpsertResolverResponse {
                success: false,
                resolver_id: self.resolver_id.clone(),
                error: Some(String::from("Failed to save and sync project symbol catalog.")),
            };
        }

        ProjectSymbolsUpsertResolverResponse {
            success: true,
            resolver_id: self.resolver_id.clone(),
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsUpsertResolverRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{project::Project, project_symbol_catalog::ProjectSymbolCatalog};
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn upsert_resolver_request_persists_resolver_and_syncs_catalog() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project = create_project_with_symbol_catalog(temp_directory.path(), ProjectSymbolCatalog::default());
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let response = ProjectSymbolsUpsertResolverRequest {
            original_resolver_id: None,
            resolver_id: String::from("inventory.count"),
            resolver_definition_json: String::from(r#"{"root_node":{"Literal":4}}"#),
        }
        .execute(&engine_execution_context);

        assert!(response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected resolver project to load from disk.");
        let resolver_descriptors = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbolic_resolver_descriptors();

        assert_eq!(resolver_descriptors.len(), 1);
        assert_eq!(resolver_descriptors[0].get_resolver_id(), "inventory.count");
    }
}
