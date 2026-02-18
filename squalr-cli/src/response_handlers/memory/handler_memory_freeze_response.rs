use squalr_engine_api::commands::memory::freeze::memory_freeze_response::MemoryFreezeResponse;

pub fn handle_memory_response_freeze(memory_freeze_response: MemoryFreezeResponse) {
    if memory_freeze_response.failed_freeze_target_count == 0 {
        log::info!("Freeze success.");
    } else {
        log::error!("Freeze failed for {} targets.", memory_freeze_response.failed_freeze_target_count);
    }
}
