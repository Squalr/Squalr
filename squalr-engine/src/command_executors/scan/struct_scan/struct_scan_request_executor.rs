use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::struct_scan::struct_scan_request::StructScanRequest;
use squalr_engine_api::commands::scan::struct_scan::struct_scan_response::StructScanResponse;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for StructScanRequest {
    type ResponseType = StructScanResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        StructScanResponse { trackable_task_handle: None }
    }
}
