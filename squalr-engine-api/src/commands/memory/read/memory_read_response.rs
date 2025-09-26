use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::memory::memory_response::MemoryResponse;
use crate::structures::structs::valued_struct::ValuedStruct;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryReadResponse {
    pub valued_struct: ValuedStruct,
    pub address: u64,
    pub success: bool,
}

impl TypedEngineCommandResponse for MemoryReadResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Memory(MemoryResponse::Read {
            memory_read_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Memory(MemoryResponse::Read { memory_read_response }) = response {
            Ok(memory_read_response)
        } else {
            Err(response)
        }
    }
}
