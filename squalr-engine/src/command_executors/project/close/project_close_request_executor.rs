use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::close::project_close_response::ProjectCloseResponse;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProjectCloseRequest {
    type ResponseType = ProjectCloseResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        engine_privileged_state
            .get_project_manager()
            .clear_opened_project();

        ProjectCloseResponse {}
    }
}
