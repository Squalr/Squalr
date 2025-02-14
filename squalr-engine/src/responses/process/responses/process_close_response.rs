use crate::responses::engine_response::EngineResponse;
use crate::responses::engine_response::TypedEngineResponse;
use crate::responses::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_processes::process_info::OpenedProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessCloseResponse {
    pub process_info: OpenedProcessInfo,
}

impl TypedEngineResponse for ProcessCloseResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::Close { process_info }) = response {
            Ok(Self { process_info })
        } else {
            Err(response)
        }
    }
}
