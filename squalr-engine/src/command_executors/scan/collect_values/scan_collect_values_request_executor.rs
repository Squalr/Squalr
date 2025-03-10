use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_scanning::scanners::value_collector_task::ValueCollectorTask;
use std::sync::Arc;
use std::thread;

impl EngineCommandRequestExecutor for ScanCollectValuesRequest {
    type ResponseType = ScanCollectValuesResponse;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = execution_context.get_opened_process() {
            let snapshot = execution_context.get_snapshot();
            let task = ValueCollectorTask::start_task(process_info.clone(), snapshot, true);
            let task_handle = task.get_task_handle();
            let progress_receiver = task.subscribe_to_progress_updates();
            let execution_context = execution_context.clone();

            execution_context.register_task(task.clone());

            // Spawn a thread to listen to progress updates
            thread::spawn(move || {
                while let Ok(progress) = progress_receiver.recv() {
                    log::info!("Progress: {:.2}%", progress);
                }
            });

            thread::spawn(move || {
                task.wait_for_completion();
                execution_context.unregister_task(&task.get_task_identifier());
                execution_context.emit_event(ScanResultsUpdatedEvent {});
            });

            ScanCollectValuesResponse {
                trackable_task_handle: Some(task_handle),
            }
        } else {
            log::error!("No opened process");
            ScanCollectValuesResponse { trackable_task_handle: None }
        }
    }
}
