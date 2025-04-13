use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use squalr_engine_api::commands::scan::reset::scan_reset_response::ScanResetResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResetRequest {
    type ResponseType = ScanResetResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let snapshot = engine_privileged_state.get_snapshot();
        let snapshot_scan_result_freeze_list = engine_privileged_state.get_snapshot_scan_result_freeze_list();

        match snapshot.write() {
            Ok(mut snapshot) => {
                // Clears snapshot regions to reset the scan.
                snapshot.set_snapshot_regions(vec![]);
                engine_privileged_state.emit_event(ScanResultsUpdatedEvent {});

                // Best-effort to clear the freeze list.
                match snapshot_scan_result_freeze_list.read() {
                    Ok(snapshot_scan_result_freeze_list) => {
                        snapshot_scan_result_freeze_list.clear();
                    }
                    Err(err) => {
                        log::error!("Failed to acquire write lock on snapshot: {}", err);
                    }
                }

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
