use crate::command_executors::engine_request_executor::EngineRequestExecutor;
use crate::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::scan::manual::scan_manual_request::ScanManualRequest;
use squalr_engine_api::commands::scan::manual::scan_manual_response::ScanManualResponse;
use squalr_engine_common::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_scanning::scan_settings::ScanSettings;
use squalr_engine_scanning::scanners::manual_scanner::ManualScanner;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use std::sync::Arc;
use std::thread;

impl EngineRequestExecutor for ScanManualRequest {
    type ResponseType = ScanManualResponse;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> <Self as EngineRequestExecutor>::ResponseType {
        if let Some(process_info) = execution_context.get_opened_process() {
            let snapshot = execution_context.get_snapshot();
            let floating_point_tolerance = ScanSettings::get_instance().get_floating_point_tolerance();
            let scan_parameters = ScanParametersGlobal::new(self.compare_type.to_owned(), self.scan_value.to_owned(), floating_point_tolerance);

            // First collect values before the manual scan.
            // TODO: This should not be blocking.
            ValueCollector::collect_values(process_info.clone(), snapshot.clone(), None, true).wait_for_completion();

            // Perform the manual scan on the collected memory.
            let task = ManualScanner::scan(snapshot, &scan_parameters, None, true);
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

            ScanManualResponse {
                trackable_task_handle: Some(task_handle),
            }
        } else {
            log::error!("No opened process");
            ScanManualResponse { trackable_task_handle: None }
        }
    }
}
