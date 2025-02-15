use crate::commands::command_handler::CommandHandler;
use crate::commands::engine_command::EngineCommand;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::process::process_request::ProcessRequest;
use crate::responses::engine_response::EngineResponse;
use crate::responses::process::process_response::ProcessResponse;
use crate::responses::process::responses::process_close_response::ProcessCloseResponse;
use crate::squalr_engine::SqualrEngine;
use crate::squalr_session::SqualrSession;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProcessCloseRequest {}

impl CommandHandler for ProcessCloseRequest {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        if let Some(process_info) = SqualrSession::get_opened_process() {
            Logger::get_instance().log(
                LogLevel::Info,
                &format!("Closing process {} with handle {}", process_info.process_id, process_info.handle),
                None,
            );

            match ProcessQuery::close_process(process_info.handle) {
                Ok(_) => {
                    SqualrSession::clear_opened_process();

                    let response = EngineResponse::Process(ProcessResponse::Close {
                        process_close_response: ProcessCloseResponse { process_info: process_info },
                    });

                    SqualrEngine::dispatch_response(response, uuid);
                }
                Err(e) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to close process handle {}: {}", process_info.handle, e), None);
                }
            }
        } else {
            Logger::get_instance().log(LogLevel::Info, "No process to close", None);
        }
    }
}

impl From<ProcessCloseResponse> for ProcessResponse {
    fn from(process_close_response: ProcessCloseResponse) -> Self {
        ProcessResponse::Close {
            process_close_response: process_close_response,
        }
    }
}

impl ProcessRequest for ProcessCloseRequest {
    type ResponseType = ProcessCloseResponse;

    fn to_command(&self) -> EngineCommand {
        EngineCommand::Process(ProcessCommand::Close {
            process_close_request: self.clone(),
        })
    }
}
