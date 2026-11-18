use super::{close::process_close_request_executor::close_process, open::process_open_request_executor::open_process};
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
        execute_process_command(self, engine_privileged_state, None)
    }
}

pub(crate) fn execute_process_command(
    command: &ProcessCommand,
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    expected_generation: Option<u64>,
) -> PrivilegedCommandResponse {
    match command {
        ProcessCommand::Open { process_open_request } => open_process(process_open_request, engine_privileged_state, expected_generation).to_engine_response(),
        ProcessCommand::List { process_list_request } => process_list_request
            .execute(engine_privileged_state)
            .to_engine_response(),
        ProcessCommand::Icon { process_icon_request } => process_icon_request
            .execute(engine_privileged_state)
            .to_engine_response(),
        ProcessCommand::Close { .. } => close_process(engine_privileged_state, expected_generation).to_engine_response(),
    }
}
