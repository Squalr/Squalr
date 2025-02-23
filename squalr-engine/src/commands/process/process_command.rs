use crate::commands::engine_request::EngineRequest;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::process::list::process_list_request::ProcessListRequest;
use crate::commands::process::open::process_open_request::ProcessOpenRequest;
use crate::commands::{engine_response::EngineResponse, process::close::process_close_request::ProcessCloseRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProcessCommand {
    Open {
        #[structopt(flatten)]
        process_open_request: ProcessOpenRequest,
    },
    List {
        #[structopt(flatten)]
        process_list_request: ProcessListRequest,
    },
    Close {
        #[structopt(flatten)]
        process_close_request: ProcessCloseRequest,
    },
}

impl ProcessCommand {
    pub fn execute(&self) -> EngineResponse {
        match self {
            ProcessCommand::Open { process_open_request } => process_open_request.execute().to_engine_response(),
            ProcessCommand::List { process_list_request } => process_list_request.execute().to_engine_response(),
            ProcessCommand::Close { process_close_request } => process_close_request.execute().to_engine_response(),
        }
    }
}
