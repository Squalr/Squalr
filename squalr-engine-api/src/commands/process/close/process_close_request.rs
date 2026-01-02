use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::process::close::process_close_response::ProcessCloseResponse;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProcessCloseRequest {}

impl PrivilegedCommandRequest for ProcessCloseRequest {
    type ResponseType = ProcessCloseResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Process(ProcessCommand::Close {
            process_close_request: self.clone(),
        })
    }
}

impl From<ProcessCloseResponse> for ProcessResponse {
    fn from(process_close_response: ProcessCloseResponse) -> Self {
        ProcessResponse::Close { process_close_response }
    }
}
