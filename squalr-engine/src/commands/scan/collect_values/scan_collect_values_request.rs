use crate::commands::engine_command::EngineCommand;
use crate::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_request::ScanRequest;
use crate::commands::scan::scan_response::ScanResponse;
use crate::squalr_engine::SqualrEngine;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use std::thread;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanCollectValuesRequest {}

impl ScanRequest for ScanCollectValuesRequest {
    type ResponseType = ScanCollectValuesResponse;

    fn execute(&self) -> Self::ResponseType {
        if let Some(process_info) = SqualrEngine::get_opened_process() {
            let snapshot = SqualrEngine::get_snapshot();
            let task = ValueCollector::collect_values(process_info.clone(), snapshot, None, true);

            // Spawn a thread to listen to progress updates
            let progress_receiver = task.add_listener();
            thread::spawn(move || {
                while let Ok(progress) = progress_receiver.recv() {
                    Logger::get_instance().log(LogLevel::Info, &format!("Progress: {:.2}%", progress), None);
                }
            });

            // Wait for completion synchronously
            task.wait_for_completion();
        } else {
            Logger::get_instance().log(LogLevel::Info, "No opened process", None);
        }

        ScanCollectValuesResponse {}
    }

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::CollectValues {
            scan_value_collector_request: self.clone(),
        })
    }
}

impl From<ScanCollectValuesResponse> for ScanResponse {
    fn from(scan_value_collector_response: ScanCollectValuesResponse) -> Self {
        ScanResponse::CollectValues { scan_value_collector_response }
    }
}
