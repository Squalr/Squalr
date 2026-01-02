use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::structures::scan_results::scan_result::ScanResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanResultsRefreshResponse {
    pub scan_results: Vec<ScanResult>,
}

impl TypedPrivilegedCommandResponse for ScanResultsRefreshResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Results(ScanResultsResponse::Refresh {
            scan_results_refresh_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Results(ScanResultsResponse::Refresh { scan_results_refresh_response }) = response {
            Ok(scan_results_refresh_response)
        } else {
            Err(response)
        }
    }
}
