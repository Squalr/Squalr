use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanNewResponse {}

impl TypedEngineCommandResponse for ScanNewResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Scan(ScanResponse::New {
            scan_new_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Scan(ScanResponse::New { scan_new_response }) = response {
            Ok(scan_new_response)
        } else {
            Err(response)
        }
    }
}
