use crate::command_executors::project::project_symbol_sync::sync_project_symbol_catalog;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::close::project_close_response::ProjectCloseResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectCloseRequest {
    type ResponseType = ProjectCloseResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if !sync_project_symbol_catalog(engine_unprivileged_state, ProjectSymbolCatalog::default()) {
            return ProjectCloseResponse { success: false };
        }

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();

        if let Ok(mut opened_project) = opened_project.write() {
            *opened_project = None;

            ProjectCloseResponse { success: true }
        } else {
            ProjectCloseResponse { success: false }
        }
    }
}
