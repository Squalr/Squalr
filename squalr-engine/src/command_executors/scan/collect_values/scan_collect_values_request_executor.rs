use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::scan::scan_results_metadata_collector::collect_scan_results_metadata;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use squalr_engine_scanning::scanners::value_collector_task::ValueCollector;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ScanCollectValuesRequest {
    type ResponseType = ScanCollectValuesResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            let snapshot = engine_privileged_state.get_snapshot();
            let memory_read_provider = engine_privileged_state.get_os_providers().memory_read.clone();
            let scan_execution_context = ScanExecutionContext::new(
                None,
                None,
                Some(Arc::new(move |opened_process_info, address, values| {
                    memory_read_provider.read_bytes(opened_process_info, address, values)
                })),
            );
            ValueCollector::collect_values(process_info.clone(), snapshot, true, &scan_execution_context);
            engine_privileged_state.emit_event(ScanResultsUpdatedEvent { is_new_scan: false });

            ScanCollectValuesResponse {
                scan_results_metadata: collect_scan_results_metadata(engine_privileged_state),
            }
        } else {
            log::error!("No opened process");
            ScanCollectValuesResponse::default()
        }
    }
}
