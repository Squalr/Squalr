use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::structures::memory::{normalized_module::NormalizedModule, normalized_region::NormalizedRegion};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryQueryResponse {
    pub virtual_pages: Vec<NormalizedRegion>,
    pub modules: Vec<NormalizedModule>,
    pub success: bool,
}

impl TypedPrivilegedCommandResponse for MemoryQueryResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Memory(MemoryResponse::Query {
            memory_query_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Memory(MemoryResponse::Query { memory_query_response }) = response {
            Ok(memory_query_response)
        } else {
            Err(response)
        }
    }
}
