use crate::responses::engine_response::EngineResponse;
use crate::responses::engine_response::TypedEngineResponse;
use crate::responses::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanManualResponse {}

impl TypedEngineResponse for ScanManualResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Scan(ScanResponse::Manual { scan_manual_response }) = response {
            Ok(scan_manual_response)
        } else {
            Err(response)
        }
    }
}
