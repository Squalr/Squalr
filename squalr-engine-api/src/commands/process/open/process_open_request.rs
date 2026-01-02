use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::process::open::process_open_response::ProcessOpenResponse;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProcessOpenRequest {
    #[structopt(short = "p", long)]
    pub process_id: Option<u32>,
    #[structopt(short = "n", long)]
    pub search_name: Option<String>,
    #[structopt(short = "m", long)]
    pub match_case: bool,
}

impl PrivilegedCommandRequest for ProcessOpenRequest {
    type ResponseType = ProcessOpenResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Process(ProcessCommand::Open {
            process_open_request: self.clone(),
        })
    }
}

impl From<ProcessOpenResponse> for ProcessResponse {
    fn from(process_open_response: ProcessOpenResponse) -> Self {
        ProcessResponse::Open { process_open_response }
    }
}
