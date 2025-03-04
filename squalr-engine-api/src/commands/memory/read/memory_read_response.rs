use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::memory::memory_response::MemoryResponse;
use serde::{Deserialize, Serialize};
// use squalr_engine_common::structures::dynamic_struct::dynamic_struct::DynamicStruct;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryReadResponse {
    // pub value: DynamicStruct,
    pub address: u64,
    pub success: bool,
}

impl TypedEngineResponse for MemoryReadResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Memory(MemoryResponse::Read {
            memory_read_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Memory(MemoryResponse::Read { memory_read_response }) = response {
            Ok(memory_read_response)
        } else {
            Err(response)
        }
    }
}
