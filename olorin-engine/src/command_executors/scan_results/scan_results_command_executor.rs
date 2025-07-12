use crate::command_executors::engine_command_executor::EngineCommandExecutor;
use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::engine_command_response::{EngineCommandResponse, TypedEngineCommandResponse};
use olorin_engine_api::commands::scan_results::scan_results_command::ScanResultsCommand;
use std::sync::Arc;

impl EngineCommandExecutor for ScanResultsCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
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
            ScanResultsCommand::AddToProject {
                results_add_to_project_request,
            } => results_add_to_project_request
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
