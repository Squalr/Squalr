use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::trackable_tasks::list::trackable_tasks_list_request::TrackableTasksListRequest;
use squalr_engine_api::commands::trackable_tasks::list::trackable_tasks_list_response::TrackableTasksListResponse;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for TrackableTasksListRequest {
    type ResponseType = TrackableTasksListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        TrackableTasksListResponse {}
    }
}
