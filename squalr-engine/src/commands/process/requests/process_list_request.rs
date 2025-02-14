use crate::commands::command_handler::CommandHandler;
use crate::commands::engine_command::EngineCommand;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::request_sender::RequestSender;
use crate::responses::engine_response::EngineResponse;
use crate::responses::process::process_response::ProcessResponse;
use crate::responses::process::responses::process_list_response::ProcessListResponse;
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
        let response = EngineResponse::Process(ProcessResponse::List { processes: processes });

        SqualrEngine::dispatch_response(response, uuid);
    }
}

impl RequestSender for ProcessListRequest {
    type ResponseType = ProcessListResponse;

    fn send<F>(
        &self,
        callback: F,
    ) where
        F: FnOnce(Self::ResponseType) + Send + Sync + 'static,
    {
        SqualrEngine::dispatch_command(self.to_command(), move |engine_response| match engine_response {
            EngineResponse::Process(process_response) => match process_response {
                ProcessResponse::List { processes } => callback(Self::ResponseType { processes }),
                _ => {}
            },
            _ => {}
        });
    }

    fn to_command(&self) -> EngineCommand {
        EngineCommand::Process(ProcessCommand::List {
            process_list_request: self.clone(),
        })
    }
}
