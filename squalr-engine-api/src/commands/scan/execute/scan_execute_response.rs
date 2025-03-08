use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::structures::tasks::trackable_task_handle::TrackableTaskHandle;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanExecuteResponse {
    pub trackable_task_handle: Option<TrackableTaskHandle>,
}

impl TypedEngineResponse for ScanExecuteResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Scan(ScanResponse::Execute {
            scan_execute_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Scan(ScanResponse::Execute { scan_execute_response }) = response {
            Ok(scan_execute_response)
        } else {
            Err(response)
        }
    }
}
