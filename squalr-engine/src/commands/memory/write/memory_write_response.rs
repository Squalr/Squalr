use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::memory::memory_response::MemoryResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryWriteResponse {
    pub success: bool,
}

impl TypedEngineResponse for MemoryWriteResponse {
    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Memory(MemoryResponse::Write { memory_write_response }) = response {
            Ok(memory_write_response)
        } else {
            Err(response)
        }
    }
}
