use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::execute::scan_execute_request::ScanExecuteRequest;
use squalr_engine_api::commands::scan::execute::scan_execute_response::ScanExecuteResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_scanning::scan_settings::ScanSettings;
use squalr_engine_scanning::scanners::scan_executor_task::ScanExecutorTask;
use std::sync::Arc;
use std::thread;

impl EngineCommandRequestExecutor for ScanExecuteRequest {
    type ResponseType = ScanExecuteResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state.get_opened_process() {
            let snapshot = engine_privileged_state.get_snapshot();
            let scan_parameters = ScanParametersGlobal::new(
                self.compare_type.to_owned(),
                self.scan_value.to_owned(),
                ScanSettings::get_instance().get_floating_point_tolerance(),
                self.memory_read_mode,
            );

            // Start the task to perform the scan.
            let task = ScanExecutorTask::start_task(process_info, snapshot, &scan_parameters, true);
            let task_handle = task.get_task_handle();
            let engine_privileged_state = engine_privileged_state.clone();
            let progress_receiver = task.subscribe_to_progress_updates();

            engine_privileged_state.register_task(task.clone());

            // Spawn a thread to listen to progress updates.
            thread::spawn(move || {
                while let Ok(progress) = progress_receiver.recv() {
                    log::info!("Progress: {:.2}%", progress);
                }
            });

            thread::spawn(move || {
                task.wait_for_completion();
                engine_privileged_state.unregister_task(&task.get_task_identifier());
                engine_privileged_state.emit_event(ScanResultsUpdatedEvent {});
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
