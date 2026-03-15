use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_response::ScanResultsDeleteResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ScanResultsDeleteRequest {
    type ResponseType = ScanResultsDeleteResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let snapshot = engine_privileged_state.get_snapshot();
        let deleted_result_count = match snapshot.write() {
            Ok(mut snapshot_guard) => snapshot_guard.delete_scan_results(
                self.scan_result_refs
                    .iter()
                    .map(|scan_result_ref| scan_result_ref.get_scan_result_global_index()),
            ),
            Err(error) => {
                log::error!("Failed to acquire write lock on snapshot for delete request: {}", error);
                return ScanResultsDeleteResponse {};
            }
        };

        if deleted_result_count > 0 {
            engine_privileged_state.emit_event(ScanResultsUpdatedEvent { is_new_scan: false });
        }

        ScanResultsDeleteResponse {}
    }
}
