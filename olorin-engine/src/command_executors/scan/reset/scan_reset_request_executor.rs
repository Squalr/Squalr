use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use olorin_engine_api::commands::scan::reset::scan_reset_response::ScanResetResponse;
use olorin_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResetRequest {
    type ResponseType = ScanResetResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let snapshot = engine_privileged_state.get_snapshot();
        let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
        let mut freeze_list_registry_guard = match freeze_list_registry.write() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire write lock on FreezeListRegistry: {}", error);

                return ScanResetResponse { success: false };
            }
        };

        match snapshot.write() {
            Ok(mut snapshot) => {
                // Clear the freeze list.
                freeze_list_registry_guard.clear();

                // Clears snapshot regions to reset the scan.
                snapshot.set_snapshot_regions(vec![]);
                engine_privileged_state.emit_event(ScanResultsUpdatedEvent { is_new_scan: false });

                log::info!("Cleared scan data.");

                ScanResetResponse { success: true }
            }
            Err(error) => {
                log::error!("Failed to acquire write lock on snapshot: {}", error);

                ScanResetResponse { success: false }
            }
        }
    }
}
