use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use squalr_engine_api::commands::scan::reset::scan_reset_response::ScanResetResponse;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResetRequest {
    type ResponseType = ScanResetResponse;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let snapshot = execution_context.get_snapshot();

        match snapshot.write() {
            Ok(mut snapshot) => {
                // Clears snapshot regions to reset the scan.
                snapshot.set_snapshot_regions(vec![]);

                log::info!("Cleared scan data.");

                ScanResetResponse { success: true }
            }
            Err(err) => {
                log::error!("Failed to acquire write lock on snapshot: {}", err);

                ScanResetResponse { success: false }
            }
        }
    }
}
