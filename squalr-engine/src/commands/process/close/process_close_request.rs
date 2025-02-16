use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::process::close::process_close_response::ProcessCloseResponse;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::process::process_response::ProcessResponse;
use crate::squalr_engine::SqualrEngine;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProcessCloseRequest {}

impl EngineRequest for ProcessCloseRequest {
    type ResponseType = ProcessCloseResponse;

    fn execute(&self) -> Self::ResponseType {
        if let Some(process_info) = SqualrEngine::get_opened_process() {
            Logger::get_instance().log(
                LogLevel::Info,
                &format!("Closing process {} with handle {}", process_info.process_id, process_info.handle),
                None,
            );

            match ProcessQuery::close_process(process_info.handle) {
                Ok(_) => {
                    SqualrEngine::clear_opened_process();
                }
                Err(err) => {
                    Logger::get_instance().log(
                        LogLevel::Error,
                        &format!("Failed to close process handle {}: {}", process_info.handle, err),
                        None,
                    );
                }
            }

            ProcessCloseResponse {
                process_info: Some(process_info),
            }
        } else {
            Logger::get_instance().log(LogLevel::Info, "No process to close", None);
            ProcessCloseResponse { process_info: None }
        }
    }

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Process(ProcessCommand::Close {
            process_close_request: self.clone(),
        })
    }
}

impl From<ProcessCloseResponse> for ProcessResponse {
    fn from(process_close_response: ProcessCloseResponse) -> Self {
        ProcessResponse::Close { process_close_response }
    }
}
