use crate::commands::memory::freeze::memory_freeze_response::MemoryFreezeResponse;
use crate::commands::memory::query::memory_query_response::MemoryQueryResponse;
use crate::commands::memory::read::memory_read_response::MemoryReadResponse;
use crate::commands::memory::write::memory_write_response::MemoryWriteResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryResponse {
    Freeze { memory_freeze_response: MemoryFreezeResponse },
    Query { memory_query_response: MemoryQueryResponse },
    Read { memory_read_response: MemoryReadResponse },
    Write { memory_write_response: MemoryWriteResponse },
}

impl MemoryResponse {
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::Freeze { memory_freeze_response: reply } if reply.failed_freeze_target_count > 0 => Some("Some freeze targets failed."),
            Self::Query { memory_query_response: reply } if !reply.success => Some("The command failed."),
            Self::Read { memory_read_response: reply } if !reply.success => Some("The command failed."),
            Self::Write { memory_write_response: reply } if !reply.success => Some(reply.error.as_deref().unwrap_or("The command failed.")),
            _ => None,
        }
    }
}
