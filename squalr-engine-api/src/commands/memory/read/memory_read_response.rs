use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::structures::structs::valued_struct::ValuedStruct;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryReadResponse {
    pub valued_struct: ValuedStruct,
    pub address: u64,
    pub success: bool,
}

impl TypedPrivilegedCommandResponse for MemoryReadResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Memory(MemoryResponse::Read {
            memory_read_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Memory(MemoryResponse::Read { memory_read_response }) = response {
            Ok(memory_read_response)
        } else {
            Err(response)
        }
    }
}
