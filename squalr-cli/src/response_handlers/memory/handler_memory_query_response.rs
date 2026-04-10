use squalr_engine_api::commands::memory::query::memory_query_response::MemoryQueryResponse;

pub fn handle_memory_query_response(memory_query_response: MemoryQueryResponse) {
    if memory_query_response.success {
        log::info!(
            "Enumerated {} virtual pages across {} modules.",
            memory_query_response.virtual_pages.len(),
            memory_query_response.modules.len()
        );
    } else {
        log::error!("Memory query failed.");
    }
}
