use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanResultsFreezeResponse {}

impl TypedEngineCommandResponse for ScanResultsFreezeResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Results(ScanResultsResponse::Freeze {
            scan_results_freeze_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Results(ScanResultsResponse::Freeze { scan_results_freeze_response }) = response {
            Ok(scan_results_freeze_response)
        } else {
            Err(response)
        }
    }
}
