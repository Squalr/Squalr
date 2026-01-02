use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::close::project_close_response::ProjectCloseResponse;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectCloseRequest {
    type ResponseType = ProjectCloseResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        /*
        let project_manger = engine_api_unprivileged_bindings.get_project_manager();
        let opened_project = self.get_opened_project();
        let mut project = opened_project
            .write()
            .map_err(|e| anyhow!("Failed to acquire write lock on opened project: {}", e))?;

        *project = None;*/

        ProjectCloseResponse {}
    }
}
