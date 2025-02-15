use crate::commands::command_handler::CommandHandler;
use crate::commands::engine_command::EngineCommand;
use crate::commands::request_sender::RequestSender;
use crate::commands::scan::scan_command::ScanCommand;
use crate::responses::engine_response::EngineResponse;
use crate::responses::scan::responses::scan_hybrid_response::ScanHybridResponse;
use crate::responses::scan::scan_response::ScanResponse;
use crate::squalr_engine::SqualrEngine;
use crate::squalr_session::SqualrSession;
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
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanManualRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<AnonymousValue>,
    #[structopt(short = "c", long)]
    pub compare_type: ScanCompareType,
}

impl CommandHandler for ScanManualRequest {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        if let Some(process_info) = SqualrSession::get_opened_process() {
            let snapshot = SqualrSession::get_snapshot();
            let scan_parameters = ScanParameters::new_with_value(self.compare_type.to_owned(), self.scan_value.to_owned());

            // First collect values before the manual scan.
            ValueCollector::collect_values(process_info.clone(), snapshot.clone(), None, true).wait_for_completion();

            // Perform the manual scan on the collected memory.
            let task = ManualScanner::scan(snapshot, &scan_parameters, None, true);

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
    }
}

impl RequestSender for ScanManualRequest {
    type ResponseType = ScanHybridResponse;

    fn send<F>(
        &self,
        callback: F,
    ) where
        F: FnOnce(Self::ResponseType) + Send + Sync + 'static,
    {
        SqualrEngine::dispatch_command(self.to_command(), move |engine_response| match engine_response {
            EngineResponse::Scan(scan_response) => match scan_response {
                ScanResponse::Hybrid { scan_hybrid_response } => callback(scan_hybrid_response),
                _ => {}
            },
            _ => {}
        });
    }

    fn to_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::Manual {
            scan_manual_request: self.clone(),
        })
    }
}
