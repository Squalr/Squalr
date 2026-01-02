use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::structures::scan_results::scan_result_ref::ScanResultRef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanResultsFreezeResponse {
    pub failed_freeze_toggle_scan_result_refs: Vec<ScanResultRef>,
}

impl TypedPrivilegedCommandResponse for ScanResultsFreezeResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Results(ScanResultsResponse::Freeze {
            scan_results_freeze_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Results(ScanResultsResponse::Freeze { scan_results_freeze_response }) = response {
            Ok(scan_results_freeze_response)
        } else {
            Err(response)
        }
    }
}
