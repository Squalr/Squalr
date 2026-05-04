use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::pointer_scan::reset::pointer_scan_reset_request::PointerScanResetRequest;
use squalr_engine_api::commands::pointer_scan::reset::pointer_scan_reset_response::PointerScanResetResponse;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for PointerScanResetRequest {
    type ResponseType = PointerScanResetResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let pointer_scan_results_store = engine_privileged_state.get_pointer_scan_results();
        match pointer_scan_results_store.write() {
            Ok(mut pointer_scan_results_guard) => {
                *pointer_scan_results_guard = None;
                log::info!("Cleared the active pointer scan results.");

                PointerScanResetResponse { success: true }
            }
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan results store: {}", error);

                PointerScanResetResponse { success: false }
            }
        }
    }
}
