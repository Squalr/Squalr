use crate::commands::engine_command::EngineCommand;
use crate::commands::scan::hybrid::scan_hybrid_response::ScanHybridResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_request::ScanRequest;
use crate::commands::scan::scan_response::ScanResponse;
use crate::squalr_engine::SqualrEngine;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::values::anonymous_value::AnonymousValue;
use squalr_engine_scanning::scanners::hybrid_scanner::HybridScanner;
use squalr_engine_scanning::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_scanning::scanners::parameters::scan_parameters::ScanParameters;
use std::thread;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanHybridRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<AnonymousValue>,
    #[structopt(short = "c", long)]
    pub compare_type: ScanCompareType,
}

impl ScanRequest for ScanHybridRequest {
    type ResponseType = ScanHybridResponse;

    fn execute(&self) -> Self::ResponseType {
        if let Some(process_info) = SqualrEngine::get_opened_process() {
            let snapshot = SqualrEngine::get_snapshot();
            let scan_parameters = ScanParameters::new_with_value(self.compare_type.to_owned(), self.scan_value.to_owned());

            // Perform the hybrid scan which simultaneously collects and scans memory.
            let task = HybridScanner::scan(process_info.clone(), snapshot, &scan_parameters, None, true);

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

        ScanHybridResponse {}
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
