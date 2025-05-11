use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsAddToProjectResponse {}

impl TypedEngineCommandResponse for ScanResultsAddToProjectResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Results(ScanResultsResponse::AddToProject {
            scan_results_add_to_project_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Results(ScanResultsResponse::AddToProject {
            scan_results_add_to_project_response,
        }) = response
        {
            Ok(scan_results_add_to_project_response)
        } else {
            Err(response)
        }
    }
}
