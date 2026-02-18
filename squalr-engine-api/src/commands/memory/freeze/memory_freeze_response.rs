use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryFreezeResponse {
    pub failed_freeze_target_count: u64,
}

impl TypedPrivilegedCommandResponse for MemoryFreezeResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Memory(MemoryResponse::Freeze {
            memory_freeze_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Memory(MemoryResponse::Freeze { memory_freeze_response }) = response {
            Ok(memory_freeze_response)
        } else {
            Err(response)
        }
    }
}
