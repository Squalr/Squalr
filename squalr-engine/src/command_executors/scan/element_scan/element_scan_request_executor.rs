use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::element_scan::element_scan_request::ElementScanRequest;
use squalr_engine_api::commands::scan::element_scan::element_scan_response::ElementScanResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::parameters::element_scan::element_scan_parameters::ElementScanParameters;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use squalr_engine_scanning::scanners::element_scan_executor_task::ElementScanExecutorTask;
use std::sync::Arc;
use std::thread;

impl EngineCommandRequestExecutor for ElementScanRequest {
    type ResponseType = ElementScanResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            let snapshot = engine_privileged_state.get_snapshot();
            let element_scan_rule_registry = engine_privileged_state.get_element_scan_rule_registry();
            let symbol_registry = engine_privileged_state.get_symbol_registry();
            let alignment = ScanSettingsConfig::get_memory_alignment().unwrap_or(MemoryAlignment::Alignment1);
            let mut scan_constraints = Vec::new();

            // Deanonymize all scan constraints against all data types.
            // For example, an immediate comparison of >= 23 could end up being a byte, float, etc.
            for anonymous_scan_constraint in &self.scan_constraints {
                for data_type_ref in &self.data_type_refs {
                    if let Some(scan_constraint) = anonymous_scan_constraint.deanonymize_constraint(data_type_ref) {
                        scan_constraints.push(scan_constraint);
                    }
                }
            }

            let scan_parameters = ElementScanParameters::new(
                scan_constraints,
                self.data_type_refs.to_owned(),
                alignment,
                ScanSettingsConfig::get_floating_point_tolerance(),
                ScanSettingsConfig::get_memory_read_mode(),
                ScanSettingsConfig::get_is_single_threaded_scan(),
                ScanSettingsConfig::get_debug_perform_validation_scan(),
            );

            // Start the task to perform the scan.
            let task = ElementScanExecutorTask::start_task(process_info, snapshot, element_scan_rule_registry, symbol_registry, scan_parameters, true);
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

            ElementScanResponse {
                trackable_task_handle: Some(task_handle),
            }
        } else {
            log::error!("No opened process");
            ElementScanResponse { trackable_task_handle: None }
        }
    }
}
