use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::pointer_scan::summary::pointer_scan_summary_request::PointerScanSummaryRequest;
use squalr_engine_api::commands::pointer_scan::summary::pointer_scan_summary_response::PointerScanSummaryResponse;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for PointerScanSummaryRequest {
    type ResponseType = PointerScanSummaryResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let pointer_scan_summary = match engine_privileged_state.get_pointer_scan_results().read() {
            Ok(pointer_scan_results_guard) => pointer_scan_results_guard
                .as_ref()
                .and_then(|pointer_scan_results| {
                    if self
                        .session_id
                        .map(|requested_session_id| requested_session_id == pointer_scan_results.get_session_id())
                        .unwrap_or(true)
                    {
                        Some(pointer_scan_results.summarize())
                    } else {
                        None
                    }
                }),
            Err(error) => {
                log::error!("Failed to acquire read lock on pointer scan results store: {}", error);

                None
            }
        };

        PointerScanSummaryResponse { pointer_scan_summary }
    }
}
