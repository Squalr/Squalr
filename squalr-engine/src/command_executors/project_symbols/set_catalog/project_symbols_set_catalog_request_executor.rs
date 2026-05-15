use crate::command_executors::project_symbols::project_symbol_store_mutation::save_and_sync_project_symbol_catalog;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::set_catalog::project_symbols_set_catalog_request::ProjectSymbolsSetCatalogRequest;
use squalr_engine_api::commands::project_symbols::set_catalog::project_symbols_set_catalog_response::ProjectSymbolsSetCatalogResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsSetCatalogRequest {
    type ResponseType = ProjectSymbolsSetCatalogResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let Some(project_symbol_catalog) = self.project_symbol_catalog.clone() else {
            return ProjectSymbolsSetCatalogResponse {
                success: false,
                error: Some(String::from("Project-symbols set-catalog request requires a symbol catalog.")),
            };
        };
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                return ProjectSymbolsSetCatalogResponse {
                    success: false,
                    error: Some(format!(
                        "Failed to acquire opened project lock for project-symbols set-catalog command: {error}"
                    )),
                };
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            return ProjectSymbolsSetCatalogResponse {
                success: false,
                error: Some(String::from("Cannot set project symbols without an opened project.")),
            };
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            return ProjectSymbolsSetCatalogResponse {
                success: false,
                error: Some(String::from(
                    "Failed to resolve opened project directory for project-symbols set-catalog command.",
                )),
            };
        };

        *opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut() = project_symbol_catalog;

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsSetCatalogResponse {
                success: false,
                error: Some(String::from("Failed to save and sync project symbol catalog.")),
            };
        }

        ProjectSymbolsSetCatalogResponse { success: true, error: None }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsSetCatalogRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim};
    use std::sync::Arc;

    #[test]
    fn set_catalog_request_persists_and_syncs_replacement_catalog() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let original_catalog = ProjectSymbolCatalog::default();
        let replacement_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("u32"),
            )],
        );
        let project = create_project_with_symbol_catalog(temp_directory.path(), original_catalog);
        let mock_project_symbols_bindings = MockProjectSymbolsBindings::new();
        let captured_project_symbol_catalogs = mock_project_symbols_bindings.captured_project_symbol_catalogs();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_project_symbols_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let response = ProjectSymbolsSetCatalogRequest::new(replacement_catalog.clone()).execute(&engine_execution_context);

        assert!(response.success);

        let opened_project = engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project_guard = opened_project
            .read()
            .expect("Expected opened project read lock in test.");
        let opened_project_catalog = opened_project_guard
            .as_ref()
            .expect("Expected opened project in test.")
            .get_project_info()
            .get_project_symbol_catalog();

        assert_eq!(opened_project_catalog.get_symbol_claims().len(), 1);
        assert_eq!(opened_project_catalog.get_symbol_claims()[0].get_display_name(), "Player");

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured project symbol catalogs lock.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims().len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims()[0].get_display_name(), "Player");
    }
}
