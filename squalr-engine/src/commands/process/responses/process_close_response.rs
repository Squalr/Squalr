use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_processes::process_info::OpenedProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessCloseResponse {
    pub process_info: OpenedProcessInfo,
}

impl TypedEngineResponse for ProcessCloseResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::Close { process_close_response }) = response {
            Ok(process_close_response)
        } else {
            Err(response)
        }
    }
}
