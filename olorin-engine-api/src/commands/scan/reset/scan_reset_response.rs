use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanResetResponse {
    pub success: bool,
}

impl TypedEngineCommandResponse for ScanResetResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Scan(ScanResponse::Reset {
            scan_reset_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Scan(ScanResponse::Reset { scan_reset_response }) = response {
            Ok(scan_reset_response)
        } else {
            Err(response)
        }
    }
}
