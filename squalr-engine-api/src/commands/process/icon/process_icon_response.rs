use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::structures::processes::process_icon::ProcessIcon;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessIconEntry {
    pub process_id: u32,
    pub process_icon: Option<ProcessIcon>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProcessIconResponse {
    pub process_icons: Vec<ProcessIconEntry>,
}

impl TypedPrivilegedCommandResponse for ProcessIconResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Process(ProcessResponse::Icon {
            process_icon_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Process(ProcessResponse::Icon { process_icon_response }) = response {
            Ok(process_icon_response)
        } else {
            Err(response)
        }
    }
}
