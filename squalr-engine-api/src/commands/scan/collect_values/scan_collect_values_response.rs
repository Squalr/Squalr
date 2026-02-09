use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::structures::scan_results::scan_results_metadata::ScanResultsMetadata;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanCollectValuesResponse {
    pub scan_results_metadata: ScanResultsMetadata,
}

impl TypedPrivilegedCommandResponse for ScanCollectValuesResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Scan(ScanResponse::CollectValues {
            scan_value_collector_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Scan(ScanResponse::CollectValues { scan_value_collector_response }) = response {
            Ok(scan_value_collector_response)
        } else {
            Err(response)
        }
    }
}
