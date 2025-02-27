use crate::command_executors::engine_command_executor::EngineCommandExecutor;
use crate::command_executors::engine_request_executor::EngineRequestExecutor;
use crate::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::engine_response::{EngineResponse, TypedEngineResponse};
use squalr_engine_api::commands::project::project_command::ProjectCommand;
use std::sync::Arc;

impl EngineCommandExecutor for ProjectCommand {
    type ResponseType = EngineResponse;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            ProjectCommand::List { project_list_request } => project_list_request
                .execute(execution_context)
                .to_engine_response(),
        }
    }
}
