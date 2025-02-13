use crate::responses::engine_response::EngineResponse;
use crate::responses::engine_response::TypedEngineResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryResponse {
    Read { value: DynamicStruct, address: u64, success: bool },
    Write { success: bool },
}

impl TypedEngineResponse for MemoryResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Memory(memory_response) = response {
            Ok(memory_response)
        } else {
            Err(response)
        }
    }
}
