use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::trackable_tasks::list::trackable_tasks_list_request::TrackableTasksListRequest;
use squalr_engine_api::commands::trackable_tasks::list::trackable_tasks_list_response::TrackableTasksListResponse;
use std::sync::Arc;

impl EngineCommandRequestExecutor for TrackableTasksListRequest {
    type ResponseType = TrackableTasksListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        TrackableTasksListResponse {}
    }
}
