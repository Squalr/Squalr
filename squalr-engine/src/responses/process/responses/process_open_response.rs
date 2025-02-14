use crate::responses::engine_response::EngineResponse;
use crate::responses::engine_response::TypedEngineResponse;
use crate::responses::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_processes::process_info::OpenedProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessOpenResponse {
    pub process_info: OpenedProcessInfo,
}

impl TypedEngineResponse for ProcessOpenResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::Open { process_info }) = response {
            Ok(Self { process_info })
        } else {
            Err(response)
        }
    }
}
