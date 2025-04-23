use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::execute::scan_execute_request::ScanExecuteRequest;
use squalr_engine_api::commands::scan::execute::scan_execute_response::ScanExecuteResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::scanning::data_value_and_alignment::DataValueAndAlignment;
use squalr_engine_api::structures::scanning::parameters::user::user_scan_parameters::UserScanParameters;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use squalr_engine_scanning::scanners::scan_executor_task::ScanExecutorTask;
use std::sync::Arc;
use std::thread;

impl EngineCommandRequestExecutor for ScanExecuteRequest {
    type ResponseType = ScanExecuteResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            let snapshot = engine_privileged_state.get_snapshot();
            let data_values_and_alignments = self
                .data_types_and_alignments
                .iter()
                .map(|data_type_and_alignment| {
                    // If a scan value was provided in the form of an anonymous value, deanonymize it into an actual value.
                    // Otherwise, fall back on a data value that just contains the data type information without value bytes.
                    let data_value = self
                        .scan_value
                        .as_ref()
                        .and_then(|scan_value| {
                            scan_value
                                .deanonymize_value(data_type_and_alignment.get_data_type())
                                .ok()
                        })
                        .unwrap_or_else(|| DataValue::new(data_type_and_alignment.get_data_type().clone(), vec![]));

                    DataValueAndAlignment::new(data_value, data_type_and_alignment.get_memory_alignment_or_default())
                })
                .collect();
            let scan_parameters = UserScanParameters::new(
                self.compare_type.to_owned(),
                data_values_and_alignments,
                ScanSettingsConfig::get_floating_point_tolerance(),
                self.memory_read_mode,
                ScanSettingsConfig::get_is_single_threaded_scan(),
                ScanSettingsConfig::get_debug_perform_validation_scan(),
            );

            // Start the task to perform the scan.
            let task = ScanExecutorTask::start_task(process_info, snapshot, &scan_parameters, true);
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

            ScanExecuteResponse {
                trackable_task_handle: Some(task_handle),
            }
        } else {
            log::error!("No opened process");
            ScanExecuteResponse { trackable_task_handle: None }
        }
    }
}
