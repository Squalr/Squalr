use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_common::tasks::trackable_task_handle::TrackableTaskHandle;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanCollectValuesResponse {
    pub trackable_task_handle: Option<TrackableTaskHandle>,
}

impl TypedEngineResponse for ScanCollectValuesResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Scan(ScanResponse::CollectValues {
            scan_value_collector_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Scan(ScanResponse::CollectValues { scan_value_collector_response }) = response {
            Ok(scan_value_collector_response)
        } else {
            Err(response)
        }
    }
}
