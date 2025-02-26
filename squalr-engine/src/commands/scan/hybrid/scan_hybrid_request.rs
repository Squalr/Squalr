use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::scan::hybrid::scan_hybrid_response::ScanHybridResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_response::ScanResponse;
use crate::engine_execution_context::EngineExecutionContext;
use serde::{Deserialize, Serialize};
use squalr_engine_common::values::anonymous_value::AnonymousValue;
use squalr_engine_scanning::scanners::hybrid_scanner::HybridScanner;
use squalr_engine_scanning::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_scanning::scanners::parameters::scan_parameters::ScanParameters;
use std::sync::Arc;
use std::thread;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanHybridRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<AnonymousValue>,
    #[structopt(short = "c", long)]
    pub compare_type: ScanCompareType,
}

impl EngineRequest for ScanHybridRequest {
    type ResponseType = ScanHybridResponse;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> Self::ResponseType {
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

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::Hybrid {
            scan_hybrid_request: self.clone(),
        })
    }
}

impl From<ScanHybridResponse> for ScanResponse {
    fn from(scan_hybrid_response: ScanHybridResponse) -> Self {
        ScanResponse::Hybrid { scan_hybrid_response }
    }
}
