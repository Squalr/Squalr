use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::scan::scan_command::ScanCommand;
use std::sync::Arc;

impl PrivilegedCommandExecutor for ScanCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            ScanCommand::Reset { scan_reset_request } => scan_reset_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ScanCommand::New { scan_new_request } => scan_new_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ScanCommand::CollectValues { scan_value_collector_request } => scan_value_collector_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ScanCommand::ElementScan { element_scan_request } => element_scan_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
