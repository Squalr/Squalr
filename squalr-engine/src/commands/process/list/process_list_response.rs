use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_processes::process_info::ProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessListResponse {
    pub processes: Vec<ProcessInfo>,
}

impl TypedEngineResponse for ProcessListResponse {
    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::List { process_list_response }) = response {
            Ok(process_list_response)
        } else {
            Err(response)
        }
    }
}
