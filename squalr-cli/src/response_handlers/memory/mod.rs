pub mod memory_response_read;
pub mod memory_response_write;

use crate::response_handlers::memory::memory_response_read::handle_memory_response_read;
use crate::response_handlers::memory::memory_response_write::handle_memory_response_write;
use squalr_engine::responses::memory::memory_response::MemoryResponse;

pub fn handle_memory_response(cmd: MemoryResponse) {
    match cmd {
        MemoryResponse::Read { .. } => handle_memory_response_read(cmd),
        MemoryResponse::Write { .. } => handle_memory_response_write(cmd),
    }
}
