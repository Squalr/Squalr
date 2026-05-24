use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::process::icon::process_icon_response::ProcessIconResponse;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessIconRequest {
    pub process_ids: Vec<u32>,
}

impl PrivilegedCommandRequest for ProcessIconRequest {
    type ResponseType = ProcessIconResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Process(ProcessCommand::Icon {
            process_icon_request: self.clone(),
        })
    }
}

impl From<ProcessIconResponse> for ProcessResponse {
    fn from(process_icon_response: ProcessIconResponse) -> Self {
        ProcessResponse::Icon { process_icon_response }
    }
}
