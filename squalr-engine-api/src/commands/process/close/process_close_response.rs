use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_common::structures::processes::process_info::OpenedProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessCloseResponse {
    pub process_info: Option<OpenedProcessInfo>,
}

impl TypedEngineResponse for ProcessCloseResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Process(ProcessResponse::Close {
            process_close_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::Close { process_close_response }) = response {
            Ok(process_close_response)
        } else {
            Err(response)
        }
    }
}
