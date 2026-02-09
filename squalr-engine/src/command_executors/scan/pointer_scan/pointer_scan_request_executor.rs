use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::scan::scan_results_metadata_collector::collect_scan_results_metadata;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::pointer_scan::pointer_scan_request::PointerScanRequest;
use squalr_engine_api::commands::scan::pointer_scan::pointer_scan_response::PointerScanResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use squalr_engine_scanning::pointer_scans::pointer_scan_executor_task::PointerScanExecutor;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for PointerScanRequest {
    type ResponseType = PointerScanResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            let symbol_registry = engine_privileged_state.get_symbol_registry();
            let symbol_registry_guard = match symbol_registry.read() {
                Ok(registry) => registry,
                Err(error) => {
                    log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                    return PointerScanResponse::default();
                }
            };
            let target_address = match symbol_registry_guard.deanonymize_value_string(&self.pointer_data_type_ref, &self.target_address) {
                Ok(data_value) => data_value,
                Err(error) => {
                    log::error!("Failed to deanonimize pointer target address: {}", error);

                    return PointerScanResponse::default();
                }
            };
            let snapshot = engine_privileged_state.get_snapshot();
            let scan_parameters = PointerScanParameters::new(
                target_address,
                self.offset_size,
                self.max_depth,
                ScanSettingsConfig::get_is_single_threaded_scan(),
                ScanSettingsConfig::get_debug_perform_validation_scan(),
            );
            let memory_read_provider = engine_privileged_state.get_os_providers().memory_read.clone();
            let scan_execution_context = ScanExecutionContext::new(
                None,
                None,
                Some(Arc::new(move |opened_process_info, address, values| {
                    memory_read_provider.read_bytes(opened_process_info, address, values)
                })),
            );
            PointerScanExecutor::execute_scan(process_info, snapshot.clone(), snapshot, scan_parameters, true, &scan_execution_context);
            engine_privileged_state.emit_event(ScanResultsUpdatedEvent { is_new_scan: false });

            PointerScanResponse {
                scan_results_metadata: collect_scan_results_metadata(engine_privileged_state),
            }
        } else {
            log::error!("No opened process");
            PointerScanResponse::default()
        }
    }
}
