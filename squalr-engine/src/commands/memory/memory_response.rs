use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::memory::read::memory_read_response::MemoryReadResponse;
use crate::commands::memory::write::memory_write_response::MemoryWriteResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryResponse {
    Read { memory_read_response: MemoryReadResponse },
    Write { memory_write_response: MemoryWriteResponse },
}

impl TypedEngineResponse for MemoryResponse {
    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Memory(memory_response) = response {
            Ok(memory_response)
        } else {
            Err(response)
        }
    }
}
