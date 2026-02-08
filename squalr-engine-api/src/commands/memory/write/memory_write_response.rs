use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryWriteResponse {
    pub success: bool,
}

impl TypedPrivilegedCommandResponse for MemoryWriteResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Memory(MemoryResponse::Write {
            memory_write_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Memory(MemoryResponse::Write { memory_write_response }) = response {
            Ok(memory_write_response)
        } else {
            Err(response)
        }
    }
}
