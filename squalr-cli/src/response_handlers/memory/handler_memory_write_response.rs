use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;

pub fn handle_memory_response_write(memory_response: MemoryWriteResponse) {
    if memory_response.success {
        log::info!("Write success.");
    } else {
        log::error!(
            "Write failed{}.",
            memory_response
                .error
                .as_deref()
                .map(|error| format!(": {}", error))
                .unwrap_or_default()
        );
    }
}
