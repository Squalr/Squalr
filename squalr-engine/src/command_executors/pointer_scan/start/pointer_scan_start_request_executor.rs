use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::pointer_scan::start::pointer_scan_start_request::PointerScanStartRequest;
use squalr_engine_api::commands::pointer_scan::start::pointer_scan_start_response::PointerScanStartResponse;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use squalr_engine_scanning::pointer_scans::pointer_scan_executor_task::PointerScanExecutor;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for PointerScanStartRequest {
    type ResponseType = PointerScanStartResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            log::error!("No opened process.");

            return PointerScanStartResponse::default();
        };

        let symbol_registry = engine_privileged_state.get_symbol_registry();
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(symbol_registry_guard) => symbol_registry_guard,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return PointerScanStartResponse::default();
            }
        };
        let target_address_data_type_ref = self.pointer_size.to_data_type_ref();
        let target_address_data_value = match symbol_registry_guard.deanonymize_value_string(&target_address_data_type_ref, &self.target_address) {
            Ok(target_address_data_value) => target_address_data_value,
            Err(error) => {
                log::error!("Failed to deanonymize pointer scan target address: {}", error);

                return PointerScanStartResponse::default();
            }
        };
        let Some(target_address) = self.pointer_size.read_address_value(&target_address_data_value) else {
            log::error!("Failed to decode pointer scan target address using pointer size {}.", self.pointer_size);

            return PointerScanStartResponse::default();
        };
        let pointer_scan_parameters = PointerScanParameters::new(
            target_address,
            self.pointer_size,
            self.offset_radius,
            self.max_depth,
            ScanSettingsConfig::get_is_single_threaded_scan(),
            ScanSettingsConfig::get_debug_perform_validation_scan(),
        );
        let modules = engine_privileged_state
            .get_os_providers()
            .memory_query
            .get_modules(&process_info);
        let snapshot = engine_privileged_state.get_snapshot();
        let memory_read_provider = engine_privileged_state.get_os_providers().memory_read.clone();
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new(move |opened_process_info, address, values| {
                memory_read_provider.read_bytes(opened_process_info, address, values)
            })),
        );
        let pointer_scan_session_id = engine_privileged_state.allocate_pointer_scan_session_id();
        let pointer_scan_session = PointerScanExecutor::execute_scan(
            process_info,
            snapshot.clone(),
            snapshot,
            pointer_scan_session_id,
            pointer_scan_parameters,
            &modules,
            true,
            &scan_execution_context,
        );
        let pointer_scan_summary = pointer_scan_session.summarize();

        match engine_privileged_state.get_pointer_scan_session().write() {
            Ok(mut pointer_scan_session_guard) => {
                *pointer_scan_session_guard = Some(pointer_scan_session);
            }
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan session store: {}", error);
            }
        }

        PointerScanStartResponse {
            pointer_scan_summary: Some(pointer_scan_summary),
        }
    }
}
