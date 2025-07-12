use crate::command_executors::engine_command_executor::EngineCommandExecutor;
use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::engine_command_response::{EngineCommandResponse, TypedEngineCommandResponse};
use olorin_engine_api::commands::process::process_command::ProcessCommand;
use std::sync::Arc;

impl EngineCommandExecutor for ProcessCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
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
