use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_response::ScanResultsDeleteResponse;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsDeleteRequest {
    type ResponseType = ScanResultsDeleteResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        ScanResultsDeleteResponse {}
    }
}
