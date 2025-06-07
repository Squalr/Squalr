use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::execute::scan_execute_request::ScanExecuteRequest;
use squalr_engine_api::commands::scan::execute::scan_execute_response::ScanExecuteResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
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
            let alignment = ScanSettingsConfig::get_memory_alignment().unwrap_or(MemoryAlignment::Alignment1);
            let data_values_and_alignments = self
                .data_type_ids
                .iter()
                .filter_map(|data_type_id| match &self.scan_value {
                    Some(anonymous_value) => match anonymous_value.deanonymize_value(data_type_id) {
                        Ok(data_value) => Some(DataValueAndAlignment::new(data_value, alignment)),
                        Err(err) => {
                            log::error!("Error mapping data value: {}", err);
                            None
                        }
                    },
                    None => None,
                })
                .collect();
            let scan_parameters = UserScanParameters::new(
                self.compare_type.to_owned(),
                data_values_and_alignments,
                ScanSettingsConfig::get_floating_point_tolerance(),
                ScanSettingsConfig::get_memory_read_mode(),
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
