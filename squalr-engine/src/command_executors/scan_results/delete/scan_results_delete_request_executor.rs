use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_response::ScanResultsDeleteResponse;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ScanResultsDeleteRequest {
    type ResponseType = ScanResultsDeleteResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        ScanResultsDeleteResponse {}
    }
}
