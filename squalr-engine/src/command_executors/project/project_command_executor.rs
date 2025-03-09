use crate::command_executors::engine_command_executor::EngineCommandExecutor;
use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::engine_command_response::{EngineCommandResponse, TypedEngineCommandResponse};
use squalr_engine_api::commands::project::project_command::ProjectCommand;
use std::sync::Arc;

impl EngineCommandExecutor for ProjectCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            ProjectCommand::List { project_list_request } => project_list_request
                .execute(execution_context)
                .to_engine_response(),
        }
    }
}
