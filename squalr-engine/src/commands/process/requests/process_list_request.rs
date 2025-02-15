use crate::commands::command_handler::CommandHandler;
use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::process::process_request::ProcessRequest;
use crate::commands::process::process_response::ProcessResponse;
use crate::commands::process::responses::process_list_response::ProcessListResponse;
use crate::squalr_engine::SqualrEngine;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_query::process_query_options::ProcessQueryOptions;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProcessListRequest {
    #[structopt(short = "w", long)]
    pub require_windowed: bool,
    #[structopt(short = "n", long)]
    pub search_name: Option<String>,
    #[structopt(short = "m", long)]
    pub match_case: bool,
    #[structopt(short = "l", long)]
    pub limit: Option<u64>,
    #[structopt(short = "i", long)]
    pub fetch_icons: bool,
}

impl CommandHandler for ProcessListRequest {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        Logger::get_instance().log(
            LogLevel::Info,
            &format!(
                "Listing processes with options: require_windowed={}, search_name={:?}, match_case={}, limit={:?}",
                self.require_windowed, self.search_name, self.match_case, self.limit
            ),
            None,
        );

        let options = ProcessQueryOptions {
            search_name: self.search_name.as_ref().cloned(),
            required_process_id: None,
            require_windowed: self.require_windowed,
            match_case: self.match_case,
            fetch_icons: self.fetch_icons,
            limit: self.limit,
        };

        let processes = ProcessQuery::get_processes(options);
        let response = EngineResponse::Process(ProcessResponse::List {
            process_list_response: ProcessListResponse { processes: processes },
        });

        SqualrEngine::dispatch_response(response, uuid);
    }
}

impl From<ProcessListResponse> for ProcessResponse {
    fn from(process_list_response: ProcessListResponse) -> Self {
        ProcessResponse::List {
            process_list_response: process_list_response,
        }
    }
}

impl ProcessRequest for ProcessListRequest {
    type ResponseType = ProcessListResponse;

    fn to_command(&self) -> EngineCommand {
        EngineCommand::Process(ProcessCommand::List {
            process_list_request: self.clone(),
        })
    }
}
