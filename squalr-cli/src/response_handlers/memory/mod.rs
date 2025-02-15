pub mod handler_memory_read_response;
pub mod handler_memory_write_response;

use crate::response_handlers::memory::handler_memory_read_response::handle_memory_read_response;
use crate::response_handlers::memory::handler_memory_write_response::handle_memory_response_write;
use squalr_engine::responses::memory::memory_response::MemoryResponse;

pub fn handle_memory_response(cmd: MemoryResponse) {
    match cmd {
        MemoryResponse::Read { .. } => handle_memory_read_response(cmd),
        MemoryResponse::Write { .. } => handle_memory_response_write(cmd),
    }
}
