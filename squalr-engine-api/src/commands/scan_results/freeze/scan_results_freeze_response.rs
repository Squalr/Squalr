use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::structures::scan_results::scan_result_ref::ScanResultRef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanResultsFreezeResponse {
    pub failed_freeze_toggle_scan_result_refs: Vec<ScanResultRef>,
}

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
