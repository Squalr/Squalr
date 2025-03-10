use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use crate::tasks::trackable_task::TrackableTask;
use squalr_engine_api::commands::scan::execute::scan_execute_request::ScanExecuteRequest;
use squalr_engine_api::commands::scan::execute::scan_execute_response::ScanExecuteResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_scanning::scan_settings::ScanSettings;
use squalr_engine_scanning::scanners::scan_executor::ScanExecutor;
use std::sync::Arc;
use std::thread;

const TASK_NAME: &'static str = "Scan Executor";

impl EngineCommandRequestExecutor for ScanExecuteRequest {
    type ResponseType = ScanExecuteResponse;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = execution_context.get_opened_process() {
            let snapshot = execution_context.get_snapshot();
            let scan_parameters = ScanParametersGlobal::new(
                self.compare_type.to_owned(),
                self.scan_value.to_owned(),
                ScanSettings::get_instance().get_floating_point_tolerance(),
                self.memory_read_mode,
            );

            // Start the task to perform the scan.
            let task = TrackableTask::<()>::create(TASK_NAME.to_string(), None);
            let task_handle = task.get_task_handle();
            ScanExecutor::scan(task_handle.clone(), process_info, snapshot, &scan_parameters, true);

            execution_context.register_task(task_handle.clone());

            // Spawn a thread to listen to progress updates.
            let progress_receiver = task.subscribe_to_progress_updates();
            thread::spawn(move || {
                while let Ok(progress) = progress_receiver.recv() {
                    log::info!("Progress: {:.2}%", progress);
                }
            });

            let execution_context = execution_context.clone();

            thread::spawn(move || {
                task.wait_for_completion();
                execution_context.unregister_task(&task.get_task_identifier());
                execution_context.emit_event(ScanResultsUpdatedEvent {});
            });

            ScanExecuteResponse {
                trackable_task_handle: Some(task_handle.to_user_handle()),
            }
        } else {
            log::error!("No opened process");
            ScanExecuteResponse { trackable_task_handle: None }
        }
    }
}
