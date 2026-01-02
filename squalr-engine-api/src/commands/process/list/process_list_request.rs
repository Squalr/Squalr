use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::process::list::process_list_response::ProcessListResponse;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

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

impl PrivilegedCommandRequest for ProcessListRequest {
    type ResponseType = ProcessListResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Process(ProcessCommand::List {
            process_list_request: self.clone(),
        })
    }
}

impl From<ProcessListResponse> for ProcessResponse {
    fn from(process_list_response: ProcessListResponse) -> Self {
        ProcessResponse::List { process_list_response }
    }
}
