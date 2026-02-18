use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::memory_command::MemoryCommand;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use std::sync::Arc;

impl PrivilegedCommandExecutor for MemoryCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            MemoryCommand::Freeze { memory_freeze_request } => memory_freeze_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            MemoryCommand::Write { memory_write_request } => memory_write_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            MemoryCommand::Read { memory_read_request } => memory_read_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
