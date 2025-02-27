use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_common::tasks::trackable_task_handle::TrackableTaskHandle;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanManualResponse {
    pub trackable_task_handle: Option<TrackableTaskHandle>,
}

impl TypedEngineResponse for ScanManualResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Scan(ScanResponse::Manual {
            scan_manual_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Scan(ScanResponse::Manual { scan_manual_response }) = response {
            Ok(scan_manual_response)
        } else {
            Err(response)
        }
    }
}
