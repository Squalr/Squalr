use crate::command_executors::engine_command_executor::EngineCommandExecutor;
use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::engine_command_response::{EngineCommandResponse, TypedEngineCommandResponse};
use squalr_engine_api::commands::memory::memory_command::MemoryCommand;
use std::sync::Arc;

impl EngineCommandExecutor for MemoryCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            MemoryCommand::Write { memory_write_request } => memory_write_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            MemoryCommand::Read { memory_read_request } => memory_read_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
