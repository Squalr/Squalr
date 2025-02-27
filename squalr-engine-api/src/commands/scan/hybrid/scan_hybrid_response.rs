use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_common::tasks::trackable_task_handle::TrackableTaskHandle;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanHybridResponse {
    pub trackable_task_handle: Option<TrackableTaskHandle>,
}

impl TypedEngineResponse for ScanHybridResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Scan(ScanResponse::Hybrid {
            scan_hybrid_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Scan(ScanResponse::Hybrid { scan_hybrid_response }) = response {
            Ok(scan_hybrid_response)
        } else {
            Err(response)
        }
    }
}
