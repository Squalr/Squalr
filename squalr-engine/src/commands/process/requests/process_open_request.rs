use crate::commands::command_handler::CommandHandler;
use crate::commands::engine_command::EngineCommand;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::request_sender::RequestSender;
use crate::responses::engine_response::EngineResponse;
use crate::responses::process::process_response::ProcessResponse;
use crate::responses::process::responses::process_open_response::ProcessOpenResponse;
use crate::squalr_engine::SqualrEngine;
use crate::squalr_session::SqualrSession;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_query::process_query_options::ProcessQueryOptions;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use structopt::StructOpt;
use sysinfo::Pid;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProcessOpenRequest {
    #[structopt(short = "p", long)]
    pub process_id: Option<u32>,
    #[structopt(short = "n", long)]
    pub search_name: Option<String>,
    #[structopt(short = "m", long)]
    pub match_case: bool,
}

impl CommandHandler for ProcessOpenRequest {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        if self.process_id.is_none() && self.search_name.is_none() {
            Logger::get_instance().log(LogLevel::Error, "Error: Neither PID nor search name provided. Cannot open process.", None);
            return;
        }

        Logger::get_instance().log(LogLevel::Info, "Opening process", None);

        let options = ProcessQueryOptions {
            search_name: self.search_name.clone(),
            required_process_id: self.process_id.map(Pid::from_u32),
            require_windowed: false,
            match_case: self.match_case,
            fetch_icons: false,
            limit: Some(1),
        };

        let processes = ProcessQuery::get_processes(options);

        if let Some(process_info) = processes.first() {
            match ProcessQuery::open_process(&process_info) {
                Ok(opened_process_info) => {
                    SqualrSession::set_opened_process(opened_process_info.clone());

                    let response = EngineResponse::Process(ProcessResponse::Open {
                        process_info: opened_process_info,
                    });

                    SqualrEngine::dispatch_response(response, uuid);
                }
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to open process {}: {}", process_info.process_id, err), None);
                }
            }
        } else {
            Logger::get_instance().log(LogLevel::Warn, "No matching process found.", None);
        }
    }
}

impl RequestSender for ProcessOpenRequest {
    type ResponseType = ProcessOpenResponse;

    fn send<F>(
        &self,
        callback: F,
    ) where
        F: FnOnce(Self::ResponseType) + Send + Sync + 'static,
    {
        SqualrEngine::dispatch_command(self.to_command(), move |engine_response| match engine_response {
            EngineResponse::Process(process_response) => match process_response {
                ProcessResponse::Open { process_info } => callback(Self::ResponseType { process_info }),
                _ => {}
            },
            _ => {}
        });
    }

    fn to_command(&self) -> EngineCommand {
        EngineCommand::Process(ProcessCommand::Open {
            process_open_request: self.clone(),
        })
    }
}
