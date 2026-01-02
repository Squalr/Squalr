use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProcessCloseResponse {
    pub process_info: Option<OpenedProcessInfo>,
}

impl TypedPrivilegedCommandResponse for ProcessCloseResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Process(ProcessResponse::Close {
            process_close_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Process(ProcessResponse::Close { process_close_response }) = response {
            Ok(process_close_response)
        } else {
            Err(response)
        }
    }
}
