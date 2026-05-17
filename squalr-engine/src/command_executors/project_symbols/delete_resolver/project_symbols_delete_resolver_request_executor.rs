use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::{
    project_symbol_catalog_persistence::save_and_sync_project_symbol_catalog, project_symbol_resolver_mutation::ProjectSymbolResolverMutation,
};
use squalr_engine_api::commands::project_symbols::delete_resolver::project_symbols_delete_resolver_request::ProjectSymbolsDeleteResolverRequest;
use squalr_engine_api::commands::project_symbols::delete_resolver::project_symbols_delete_resolver_response::ProjectSymbolsDeleteResolverResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsDeleteResolverRequest {
    type ResponseType = ProjectSymbolsDeleteResolverResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                return ProjectSymbolsDeleteResolverResponse {
                    success: false,
                    resolver_id: self.resolver_id.clone(),
                    error: Some(format!(
                        "Failed to acquire opened project lock for project-symbols delete-resolver command: {error}"
                    )),
                };
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            return ProjectSymbolsDeleteResolverResponse {
                success: false,
                resolver_id: self.resolver_id.clone(),
                error: Some(String::from("Cannot delete symbol resolver without an opened project.")),
            };
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            return ProjectSymbolsDeleteResolverResponse {
                success: false,
                resolver_id: self.resolver_id.clone(),
                error: Some(String::from(
                    "Failed to resolve opened project directory for project-symbols delete-resolver command.",
                )),
            };
        };

        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();
        if let Err(error) = ProjectSymbolResolverMutation::delete_resolver(project_symbol_catalog, &self.resolver_id) {
            return ProjectSymbolsDeleteResolverResponse {
                success: false,
                resolver_id: self.resolver_id.clone(),
                error: Some(error),
            };
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsDeleteResolverResponse {
                success: false,
                resolver_id: self.resolver_id.clone(),
                error: Some(String::from("Failed to save and sync project symbol catalog.")),
            };
        }

        ProjectSymbolsDeleteResolverResponse {
            success: true,
            resolver_id: self.resolver_id.clone(),
            error: None,
        }
    }
}
