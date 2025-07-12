use crate::{
    command_executors::{engine_command_executor::EngineCommandExecutor, engine_request_executor::EngineCommandRequestExecutor},
    engine_privileged_state::EnginePrivilegedState,
};
use olorin_engine_api::commands::{
    engine_command_response::{EngineCommandResponse, TypedEngineCommandResponse},
    trackable_tasks::trackable_tasks_command::TrackableTasksCommand,
};
use std::sync::Arc;

impl EngineCommandExecutor for TrackableTasksCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            TrackableTasksCommand::List { trackable_tasks_list_request } => trackable_tasks_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            TrackableTasksCommand::Cancel {
                trackable_tasks_cancel_request,
            } => trackable_tasks_cancel_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
