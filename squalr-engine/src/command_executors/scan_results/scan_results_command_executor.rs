use crate::command_executors::engine_command_executor::EngineCommandExecutor;
use crate::command_executors::engine_request_executor::EngineRequestExecutor;
use crate::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::engine_response::{EngineResponse, TypedEngineResponse};
use squalr_engine_api::commands::scan_results::scan_results_command::ScanResultsCommand;
use std::sync::Arc;

impl EngineCommandExecutor for ScanResultsCommand {
    type ResponseType = EngineResponse;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            ScanResultsCommand::List { results_list_request } => results_list_request
                .execute(execution_context)
                .to_engine_response(),
            ScanResultsCommand::Query { results_query_request } => results_query_request
                .execute(execution_context)
                .to_engine_response(),
            ScanResultsCommand::Refresh { results_refresh_request } => results_refresh_request
                .execute(execution_context)
                .to_engine_response(),
        }
    }
}
