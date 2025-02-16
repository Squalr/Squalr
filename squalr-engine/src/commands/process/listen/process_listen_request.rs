use crate::commands::process::listen::process_listen_response::ProcessListenResponse;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::process::process_response::ProcessResponse;
use crate::commands::{engine_command::EngineCommand, engine_request::EngineRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProcessListenRequest {}

impl EngineRequest for ProcessListenRequest {
    type ResponseType = ProcessListenResponse;

    fn execute(&self) -> Self::ResponseType {
        ProcessListenResponse { opened_process_info: None }
    }

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Process(ProcessCommand::Listen {
            process_listen_request: self.clone(),
        })
    }
}

impl From<ProcessListenResponse> for ProcessResponse {
    fn from(process_listen_response: ProcessListenResponse) -> Self {
        ProcessResponse::Listen { process_listen_response }
    }
}
