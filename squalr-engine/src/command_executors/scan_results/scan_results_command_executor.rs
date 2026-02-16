use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::scan_results::scan_results_command::ScanResultsCommand;
use std::sync::Arc;

impl PrivilegedCommandExecutor for ScanResultsCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            ScanResultsCommand::List { results_list_request } => results_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ScanResultsCommand::Query { results_query_request } => results_query_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ScanResultsCommand::Refresh { results_refresh_request } => results_refresh_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ScanResultsCommand::Freeze { results_freeze_request } => results_freeze_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ScanResultsCommand::SetProperty { results_set_property_request } => results_set_property_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ScanResultsCommand::Delete { results_delete_request } => results_delete_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
