use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_common::structures::process_info::OpenedProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessOpenResponse {
    pub opened_process_info: Option<OpenedProcessInfo>,
}

impl TypedEngineResponse for ProcessOpenResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Process(ProcessResponse::Open {
            process_open_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::Open { process_open_response }) = response {
            Ok(process_open_response)
        } else {
            Err(response)
        }
    }
}
