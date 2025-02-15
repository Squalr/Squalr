use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanCollectValuesResponse {}

impl TypedEngineResponse for ScanCollectValuesResponse {
    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Scan(ScanResponse::CollectValues { scan_value_collector_response }) = response {
            Ok(scan_value_collector_response)
        } else {
            Err(response)
        }
    }
}
