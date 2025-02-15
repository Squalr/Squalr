use crate::responses::engine_response::EngineResponse;
use crate::responses::engine_response::TypedEngineResponse;
use crate::responses::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_processes::process_info::OpenedProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessOpenResponse {
    pub opened_process_info: OpenedProcessInfo,
}

impl TypedEngineResponse for ProcessOpenResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::Open { process_open_response }) = response {
            Ok(process_open_response)
        } else {
            Err(response)
        }
    }
}
