use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::process::process_command::ProcessCommand;
use std::sync::Arc;

impl PrivilegedCommandExecutor for ProcessCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            ProcessCommand::Open { process_open_request } => process_open_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ProcessCommand::List { process_list_request } => process_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ProcessCommand::Close { process_close_request } => process_close_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
