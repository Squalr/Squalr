use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResetResponse {
    pub success: bool,
}

impl TypedEngineResponse for ScanResetResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Scan(ScanResponse::Reset {
            scan_reset_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Scan(ScanResponse::Reset { scan_reset_response }) = response {
            Ok(scan_reset_response)
        } else {
            Err(response)
        }
    }
}
