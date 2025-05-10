use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsSetPropertyResponse {}

impl TypedEngineCommandResponse for ScanResultsSetPropertyResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Results(ScanResultsResponse::SetProperty {
            scan_results_set_property_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Results(ScanResultsResponse::SetProperty {
            scan_results_set_property_response,
        }) = response
        {
            Ok(scan_results_set_property_response)
        } else {
            Err(response)
        }
    }
}
