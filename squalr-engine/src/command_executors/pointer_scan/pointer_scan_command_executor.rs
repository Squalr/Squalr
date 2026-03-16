use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use std::sync::Arc;

impl PrivilegedCommandExecutor for PointerScanCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            PointerScanCommand::Reset { pointer_scan_reset_request } => pointer_scan_reset_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PointerScanCommand::Start { pointer_scan_start_request } => pointer_scan_start_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PointerScanCommand::Summary { pointer_scan_summary_request } => pointer_scan_summary_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PointerScanCommand::Expand { pointer_scan_expand_request } => pointer_scan_expand_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PointerScanCommand::Validate { pointer_scan_validate_request } => pointer_scan_validate_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
