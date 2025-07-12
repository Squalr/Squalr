pub mod handler_memory_read_response;
pub mod handler_memory_write_response;

use crate::response_handlers::memory::handler_memory_read_response::handle_memory_read_response;
use crate::response_handlers::memory::handler_memory_write_response::handle_memory_response_write;
use olorin_engine_api::commands::memory::memory_response::MemoryResponse;

pub fn handle_memory_response(cmd: MemoryResponse) {
    match cmd {
        MemoryResponse::Read { memory_read_response } => handle_memory_read_response(memory_read_response),
        MemoryResponse::Write { memory_write_response } => handle_memory_response_write(memory_write_response),
    }
}
