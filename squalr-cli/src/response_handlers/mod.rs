mod memory;
mod process;

use memory::handle_memory_response;
use process::handle_process_response;
use squalr_engine::responses::engine_response::EngineResponse;

pub fn handle_engine_response(response: EngineResponse) {
    match response {
        EngineResponse::Memory(response) => handle_memory_response(response),
        EngineResponse::Process(response) => handle_process_response(response),
    }
}
