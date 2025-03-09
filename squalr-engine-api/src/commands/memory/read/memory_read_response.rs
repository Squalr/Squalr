use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::memory::memory_response::MemoryResponse;
use serde::{Deserialize, Serialize};
// use squalr_engine_api::structures::dynamic_struct::dynamic_struct::DynamicStruct;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryReadResponse {
    // pub value: DynamicStruct,
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
