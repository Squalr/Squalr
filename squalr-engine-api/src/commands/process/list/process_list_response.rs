use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::structures::processes::process_info::ProcessInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessListResponse {
    pub processes: Vec<ProcessInfo>,
}

impl TypedEngineCommandResponse for ProcessListResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Process(ProcessResponse::List {
            process_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Process(ProcessResponse::List { process_list_response }) = response {
            Ok(process_list_response)
        } else {
            Err(response)
        }
    }
}
