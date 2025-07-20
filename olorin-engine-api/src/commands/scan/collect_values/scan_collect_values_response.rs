use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::structures::tasks::trackable_task_handle::TrackableTaskHandle;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanCollectValuesResponse {
    pub trackable_task_handle: Option<TrackableTaskHandle>,
}

impl TypedEngineCommandResponse for ScanCollectValuesResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Scan(ScanResponse::CollectValues {
            scan_value_collector_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Scan(ScanResponse::CollectValues { scan_value_collector_response }) = response {
            Ok(scan_value_collector_response)
        } else {
            Err(response)
        }
    }
}
