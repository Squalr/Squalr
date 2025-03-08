use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::structures::scan_results::scan_result::ScanResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsRefreshResponse {
    pub scan_results: Vec<ScanResult>,
}

impl TypedEngineResponse for ScanResultsRefreshResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Results(ScanResultsResponse::Refresh {
            scan_results_refresh_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Results(ScanResultsResponse::Refresh { scan_results_refresh_response }) = response {
            Ok(scan_results_refresh_response)
        } else {
            Err(response)
        }
    }
}
