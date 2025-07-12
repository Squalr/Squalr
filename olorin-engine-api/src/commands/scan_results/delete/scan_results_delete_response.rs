use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsDeleteResponse {}

impl TypedEngineCommandResponse for ScanResultsDeleteResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Results(ScanResultsResponse::Delete {
            scan_results_delete_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Results(ScanResultsResponse::Delete { scan_results_delete_response }) = response {
            Ok(scan_results_delete_response)
        } else {
            Err(response)
        }
    }
}
