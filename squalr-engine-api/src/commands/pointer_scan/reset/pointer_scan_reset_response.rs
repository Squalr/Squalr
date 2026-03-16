use crate::commands::pointer_scan::pointer_scan_response::PointerScanResponse;
use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PointerScanResetResponse {
    pub success: bool,
}

impl TypedPrivilegedCommandResponse for PointerScanResetResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::PointerScan(PointerScanResponse::Reset {
            pointer_scan_reset_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::PointerScan(PointerScanResponse::Reset { pointer_scan_reset_response }) = response {
            Ok(pointer_scan_reset_response)
        } else {
            Err(response)
        }
    }
}
