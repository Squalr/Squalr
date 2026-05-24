use crate::commands::memory::freeze::memory_freeze_request::MemoryFreezeRequest;
use crate::commands::memory::query::memory_query_request::MemoryQueryRequest;
use crate::commands::memory::read::memory_read_request::MemoryReadRequest;
use crate::commands::memory::write::memory_write_request::MemoryWriteRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryCommand {
    Freeze { memory_freeze_request: MemoryFreezeRequest },
    Query { memory_query_request: MemoryQueryRequest },
    Read { memory_read_request: MemoryReadRequest },
    Write { memory_write_request: MemoryWriteRequest },
}
