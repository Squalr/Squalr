use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::scan::scan_results_metadata_collector::collect_scan_results_metadata;
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
        StructScanResponse {
            scan_results_metadata: collect_scan_results_metadata(engine_privileged_state),
        }
    }
}
