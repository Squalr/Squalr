use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::struct_scan::struct_scan_command::StructScanCommand;
use std::sync::Arc;

impl PrivilegedCommandExecutor for StructScanCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        self.struct_scan_request
            .execute(engine_privileged_state)
            .to_engine_response()
    }
}
