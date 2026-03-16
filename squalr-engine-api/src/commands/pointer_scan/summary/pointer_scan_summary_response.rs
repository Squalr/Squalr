use crate::commands::pointer_scan::pointer_scan_response::PointerScanResponse;
use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PointerScanSummaryResponse {
    pub pointer_scan_summary: Option<PointerScanSummary>,
}

impl TypedPrivilegedCommandResponse for PointerScanSummaryResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::PointerScan(PointerScanResponse::Summary {
            pointer_scan_summary_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::PointerScan(PointerScanResponse::Summary { pointer_scan_summary_response }) = response {
            Ok(pointer_scan_summary_response)
        } else {
            Err(response)
        }
    }
}
