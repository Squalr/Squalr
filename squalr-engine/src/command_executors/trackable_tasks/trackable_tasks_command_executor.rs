use crate::{
    command_executors::{privileged_command_executor::PrivilegedCommandExecutor, privileged_request_executor::PrivilegedCommandRequestExecutor},
    engine_privileged_state::EnginePrivilegedState,
};
use squalr_engine_api::commands::{
    privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse},
    trackable_tasks::trackable_tasks_command::TrackableTasksCommand,
};
use std::sync::Arc;

impl PrivilegedCommandExecutor for TrackableTasksCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
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
