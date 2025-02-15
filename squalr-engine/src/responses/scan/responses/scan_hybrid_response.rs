use crate::responses::engine_response::EngineResponse;
use crate::responses::engine_response::TypedEngineResponse;
use crate::responses::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanHybridResponse {}

impl TypedEngineResponse for ScanHybridResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Scan(ScanResponse::Hybrid { scan_hybrid_response }) = response {
            Ok(scan_hybrid_response)
        } else {
            Err(response)
        }
    }
}
