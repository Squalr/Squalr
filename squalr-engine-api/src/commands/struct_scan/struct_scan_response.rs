use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::structures::scan_results::scan_results_metadata::ScanResultsMetadata;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StructScanResponse {
    pub scan_results_metadata: ScanResultsMetadata,
}

impl TypedPrivilegedCommandResponse for StructScanResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::StructScan(self.clone())
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::StructScan(struct_scan_response) = response {
            Ok(struct_scan_response)
        } else {
            Err(response)
        }
    }
}
