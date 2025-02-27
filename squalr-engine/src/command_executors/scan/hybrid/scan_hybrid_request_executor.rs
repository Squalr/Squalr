use crate::command_executors::engine_request_executor::EngineRequestExecutor;
use crate::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::scan::hybrid::scan_hybrid_request::ScanHybridRequest;
use squalr_engine_api::commands::scan::hybrid::scan_hybrid_response::ScanHybridResponse;
use squalr_engine_scanning::scanners::hybrid_scanner::HybridScanner;
use squalr_engine_scanning::scanners::parameters::scan_parameters::ScanParameters;
use std::sync::Arc;
use std::thread;

impl EngineRequestExecutor for ScanHybridRequest {
    type ResponseType = ScanHybridResponse;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> <Self as EngineRequestExecutor>::ResponseType {
        if let Some(process_info) = execution_context.get_opened_process() {
            let snapshot = execution_context.get_snapshot();
            let scan_parameters = ScanParameters::new_with_value(self.compare_type.to_owned(), self.scan_value.to_owned());

            // Perform the hybrid scan which simultaneously collects and scans memory.
            let task = HybridScanner::scan(process_info.clone(), snapshot, &scan_parameters, None, true);
            let task_handle = task.get_task_handle();

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

            ScanHybridResponse {
                trackable_task_handle: Some(task_handle),
            }
        } else {
            log::error!("No opened process");
            ScanHybridResponse { trackable_task_handle: None }
        }
    }
}
