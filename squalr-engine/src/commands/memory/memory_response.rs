use crate::commands::memory::read::memory_read_response::MemoryReadResponse;
use crate::commands::memory::write::memory_write_response::MemoryWriteResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryResponse {
    Read { memory_read_response: MemoryReadResponse },
    Write { memory_write_response: MemoryWriteResponse },
}
