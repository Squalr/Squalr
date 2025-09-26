use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::trackable_tasks::cancel::trackable_tasks_cancel_request::TrackableTasksCancelRequest;
use squalr_engine_api::commands::trackable_tasks::cancel::trackable_tasks_cancel_response::TrackableTasksCancelResponse;
use std::sync::Arc;

impl EngineCommandRequestExecutor for TrackableTasksCancelRequest {
    type ResponseType = TrackableTasksCancelResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        engine_privileged_state
            .get_trackable_task_manager()
            .cancel_task(&self.task_id);

        TrackableTasksCancelResponse {}
    }
}
