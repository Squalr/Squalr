use crate::commands::engine_command::EngineCommand;
use crate::commands::process::open::process_open_response::ProcessOpenResponse;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::process::process_request::ProcessRequest;
use crate::commands::process::process_response::ProcessResponse;
use crate::squalr_session::SqualrSession;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_query::process_query_options::ProcessQueryOptions;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use structopt::StructOpt;
use sysinfo::Pid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProcessOpenRequest {
    #[structopt(short = "p", long)]
    pub process_id: Option<u32>,
    #[structopt(short = "n", long)]
    pub search_name: Option<String>,
    #[structopt(short = "m", long)]
    pub match_case: bool,
}

impl ProcessRequest for ProcessOpenRequest {
    type ResponseType = ProcessOpenResponse;

    fn execute(&self) -> Self::ResponseType {
        if self.process_id.is_none() && self.search_name.is_none() {
            Logger::get_instance().log(LogLevel::Error, "Error: Neither PID nor search name provided. Cannot open process.", None);
            return ProcessOpenResponse { opened_process_info: None };
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

                    return ProcessOpenResponse {
                        opened_process_info: Some(opened_process_info),
                    };
                }
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to open process {}: {}", process_info.process_id, err), None);
                }
            }
        } else {
            Logger::get_instance().log(LogLevel::Warn, "No matching process found.", None);
        }

        ProcessOpenResponse { opened_process_info: None }
    }

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Process(ProcessCommand::Open {
            process_open_request: self.clone(),
        })
    }
}

impl From<ProcessOpenResponse> for ProcessResponse {
    fn from(process_open_response: ProcessOpenResponse) -> Self {
        ProcessResponse::Open { process_open_response }
    }
}
