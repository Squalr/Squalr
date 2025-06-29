use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::struct_scan::struct_scan_request::StructScanRequest;
use squalr_engine_api::commands::scan::struct_scan::struct_scan_response::StructScanResponse;
use std::sync::Arc;

impl EngineCommandRequestExecutor for StructScanRequest {
    type ResponseType = StructScanResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        StructScanResponse { trackable_task_handle: None }
    }
}
