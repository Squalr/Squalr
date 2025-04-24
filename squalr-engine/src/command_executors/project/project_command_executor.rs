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
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            ProjectCommand::Create { project_create_request } => project_create_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ProjectCommand::Open { project_open_request } => project_open_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ProjectCommand::Rename { project_rename_request } => project_rename_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ProjectCommand::Save { project_save_request } => project_save_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ProjectCommand::List { project_list_request } => project_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
