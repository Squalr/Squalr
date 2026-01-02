use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanResetResponse {
    pub success: bool,
}

impl TypedPrivilegedCommandResponse for ScanResetResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Scan(ScanResponse::Reset {
            scan_reset_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Scan(ScanResponse::Reset { scan_reset_response }) = response {
            Ok(scan_reset_response)
        } else {
            Err(response)
        }
    }
}
