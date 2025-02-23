use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::scan::manual::scan_manual_response::ScanManualResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_response::ScanResponse;
use crate::squalr_engine::SqualrEngine;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::values::anonymous_value::AnonymousValue;
use squalr_engine_scanning::scanners::manual_scanner::ManualScanner;
use squalr_engine_scanning::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_scanning::scanners::parameters::scan_parameters::ScanParameters;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use std::thread;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanManualRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<AnonymousValue>,
    #[structopt(short = "c", long)]
    pub compare_type: ScanCompareType,
}

impl EngineRequest for ScanManualRequest {
    type ResponseType = ScanManualResponse;

    fn execute(&self) -> Self::ResponseType {
        if let Some(process_info) = SqualrEngine::get_opened_process() {
            let snapshot = SqualrEngine::get_snapshot();
            let scan_parameters = ScanParameters::new_with_value(self.compare_type.to_owned(), self.scan_value.to_owned());

            // First collect values before the manual scan.
            ValueCollector::collect_values(process_info.clone(), snapshot.clone(), None, true).wait_for_completion();

            // Perform the manual scan on the collected memory.
            let task = ManualScanner::scan(snapshot, &scan_parameters, None, true);

            // Spawn a thread to listen to progress updates
            let progress_receiver = task.subscribe_to_progress_updates();
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

        ScanManualResponse {}
    }

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::Manual {
            scan_manual_request: self.clone(),
        })
    }
}

impl From<ScanManualResponse> for ScanResponse {
    fn from(scan_manual_response: ScanManualResponse) -> Self {
        ScanResponse::Manual { scan_manual_response }
    }
}
