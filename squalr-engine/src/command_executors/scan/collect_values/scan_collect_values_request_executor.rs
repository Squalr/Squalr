use crate::command_executors::engine_request_executor::EngineRequestExecutor;
use crate::engine_execution_context::EngineExecutionContext;
use crate::tasks::trackable_task::TrackableTask;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use std::sync::Arc;
use std::thread;

const TASK_NAME: &'static str = "Value Collector";

impl EngineRequestExecutor for ScanCollectValuesRequest {
    type ResponseType = ScanCollectValuesResponse;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> <Self as EngineRequestExecutor>::ResponseType {
        if let Some(process_info) = execution_context.get_opened_process() {
            let snapshot = execution_context.get_snapshot();
            let task = TrackableTask::<()>::create(TASK_NAME.to_string(), None);
            let task_handle = task.get_task_handle();

            ValueCollector::collect_values(task_handle.clone(), process_info.clone(), snapshot, true);

            execution_context.register_task(task_handle.clone());

            // Spawn a thread to listen to progress updates
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
            });

            ScanCollectValuesResponse {
                trackable_task_handle: Some(task_handle.to_user_handle()),
            }
        } else {
            log::error!("No opened process");
            ScanCollectValuesResponse { trackable_task_handle: None }
        }
    }
}
