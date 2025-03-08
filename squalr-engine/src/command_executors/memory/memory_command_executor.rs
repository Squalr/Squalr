use crate::command_executors::engine_command_executor::EngineCommandExecutor;
use crate::command_executors::engine_request_executor::EngineRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::engine_response::{EngineResponse, TypedEngineResponse};
use squalr_engine_api::commands::memory::memory_command::MemoryCommand;
use std::sync::Arc;

impl EngineCommandExecutor for MemoryCommand {
    type ResponseType = EngineResponse;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            MemoryCommand::Write { memory_write_request } => memory_write_request
                .execute(execution_context)
                .to_engine_response(),
            MemoryCommand::Read { memory_read_request } => memory_read_request
                .execute(execution_context)
                .to_engine_response(),
        }
    }
}
