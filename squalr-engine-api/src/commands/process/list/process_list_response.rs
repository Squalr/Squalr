use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::structures::processes::process_info::ProcessInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessListResponse {
    pub processes: Vec<ProcessInfo>,
}

impl TypedEngineResponse for ProcessListResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Process(ProcessResponse::List {
            process_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::List { process_list_response }) = response {
            Ok(process_list_response)
        } else {
            Err(response)
        }
    }
}
