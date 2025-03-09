use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProjectListRequest {
    type ResponseType = ProjectListResponse;

    fn execute(
        &self,
        _execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        // JIRA: Implement me
        ProjectListResponse {}
    }
}
