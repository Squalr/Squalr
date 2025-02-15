use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_processes::process_info::OpenedProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessListenResponse {
    pub opened_process_info: Option<OpenedProcessInfo>,
}

impl TypedEngineResponse for ProcessListenResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Process(ProcessResponse::Listen {
            process_listen_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::Listen { process_listen_response }) = response {
            Ok(process_listen_response)
        } else {
            Err(response)
        }
    }
}
