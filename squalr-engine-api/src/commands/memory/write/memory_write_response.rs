use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::memory::memory_response::MemoryResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryWriteResponse {
    pub success: bool,
}

impl TypedEngineCommandResponse for MemoryWriteResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Memory(MemoryResponse::Write {
            memory_write_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Memory(MemoryResponse::Write { memory_write_response }) = response {
            Ok(memory_write_response)
        } else {
            Err(response)
        }
    }
}
