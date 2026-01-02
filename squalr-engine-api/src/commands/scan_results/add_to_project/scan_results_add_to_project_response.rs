use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanResultsAddToProjectResponse {}

impl TypedPrivilegedCommandResponse for ScanResultsAddToProjectResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Results(ScanResultsResponse::AddToProject {
            scan_results_add_to_project_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Results(ScanResultsResponse::AddToProject {
            scan_results_add_to_project_response,
        }) = response
        {
            Ok(scan_results_add_to_project_response)
        } else {
            Err(response)
        }
    }
}
