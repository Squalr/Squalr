use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;

pub fn handle_memory_read_response(memory_read_response: MemoryReadResponse) {
    if memory_read_response.success {
        log::info!(
            "Read value {:?} from address {}",
            memory_read_response.valued_struct,
            memory_read_response.address
        );
    } else {
        log::error!("Failed to read memory");
    }
}
