use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessOpenResponse {
    pub opened_process_info: Option<OpenedProcessInfo>,
}

impl TypedPrivilegedCommandResponse for ProcessOpenResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Process(ProcessResponse::Open {
            process_open_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Process(ProcessResponse::Open { process_open_response }) = response {
            Ok(process_open_response)
        } else {
            Err(response)
        }
    }
}
