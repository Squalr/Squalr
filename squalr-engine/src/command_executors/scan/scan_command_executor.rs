use crate::command_executors::engine_command_executor::EngineCommandExecutor;
use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::engine_command_response::{EngineCommandResponse, TypedEngineCommandResponse};
use squalr_engine_api::commands::scan::scan_command::ScanCommand;
use std::sync::Arc;

impl EngineCommandExecutor for ScanCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
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
            ScanCommand::PointerScan { pointer_scan_request } => pointer_scan_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ScanCommand::StructScan { struct_scan_request } => struct_scan_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
