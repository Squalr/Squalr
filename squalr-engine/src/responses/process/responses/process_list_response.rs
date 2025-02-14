use crate::responses::engine_response::EngineResponse;
use crate::responses::engine_response::TypedEngineResponse;
use crate::responses::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_processes::process_info::ProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessListResponse {
    pub processes: Vec<ProcessInfo>,
}

impl TypedEngineResponse for ProcessListResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::List { processes }) = response {
            Ok(Self { processes })
        } else {
            Err(response)
        }
    }
}
