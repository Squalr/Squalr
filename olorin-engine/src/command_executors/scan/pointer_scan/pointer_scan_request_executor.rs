use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::scan::pointer_scan::pointer_scan_request::PointerScanRequest;
use olorin_engine_api::commands::scan::pointer_scan::pointer_scan_response::PointerScanResponse;
use olorin_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use olorin_engine_api::structures::scanning::parameters::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use olorin_engine_scanning::pointer_scans::pointer_scan_executor_task::PointerScanExecutorTask;
use olorin_engine_scanning::scan_settings_config::ScanSettingsConfig;
use std::sync::Arc;
use std::thread;

impl EngineCommandRequestExecutor for PointerScanRequest {
    type ResponseType = PointerScanResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
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
            let target_address = match symbol_registry_guard.deanonymize_value(&self.pointer_data_type_ref, self.target_address.get_value()) {
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

            // Start the task to perform the scan.
            let element_scan_rule_registry = engine_privileged_state.get_element_scan_rule_registry();
            let symbol_registry = engine_privileged_state.get_symbol_registry();
            let task = PointerScanExecutorTask::start_task(
                process_info,
                snapshot.clone(),
                snapshot.clone(),
                element_scan_rule_registry,
                symbol_registry,
                scan_parameters,
                true,
            );
            let task_handle = task.get_task_handle();
            let engine_privileged_state = engine_privileged_state.clone();
            let progress_receiver = task.subscribe_to_progress_updates();

            engine_privileged_state
                .get_trackable_task_manager()
                .register_task(task.clone());

            // Spawn a thread to listen to progress updates.
            thread::spawn(move || {
                while let Ok(progress) = progress_receiver.recv() {
                    log::info!("Progress: {:.2}%", progress);
                }
            });

            thread::spawn(move || {
                task.wait_for_completion();
                engine_privileged_state
                    .get_trackable_task_manager()
                    .unregister_task(&task.get_task_identifier());
                engine_privileged_state.emit_event(ScanResultsUpdatedEvent { is_new_scan: false });
            });

            PointerScanResponse {
                trackable_task_handle: Some(task_handle),
            }
        } else {
            log::error!("No opened process");
            PointerScanResponse { trackable_task_handle: None }
        }
    }
}
