use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::structures::scan_results::scan_result::ScanResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanResultsRefreshResponse {
    pub scan_results: Vec<ScanResult>,
}

impl TypedEngineCommandResponse for ScanResultsRefreshResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Results(ScanResultsResponse::Refresh {
            scan_results_refresh_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Results(ScanResultsResponse::Refresh { scan_results_refresh_response }) = response {
            Ok(scan_results_refresh_response)
        } else {
            Err(response)
        }
    }
}
