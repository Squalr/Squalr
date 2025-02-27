use crate::command_executors::engine_command_executor::EngineCommandExecutor;
use crate::command_executors::engine_request_executor::EngineRequestExecutor;
use crate::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::engine_response::{EngineResponse, TypedEngineResponse};
use squalr_engine_api::commands::process::process_command::ProcessCommand;
use std::sync::Arc;

impl EngineCommandExecutor for ProcessCommand {
    type ResponseType = EngineResponse;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            ProcessCommand::Open { process_open_request } => process_open_request
                .execute(execution_context)
                .to_engine_response(),
            ProcessCommand::List { process_list_request } => process_list_request
                .execute(execution_context)
                .to_engine_response(),
            ProcessCommand::Close { process_close_request } => process_close_request
                .execute(execution_context)
                .to_engine_response(),
        }
    }
}
