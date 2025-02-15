use squalr_engine::responses::memory::memory_response::MemoryResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_memory_response_write(memory_response: MemoryResponse) {
    if let MemoryResponse::Write { success } = memory_response {
        if success {
            Logger::get_instance().log(LogLevel::Info, "Write success.", None);
        } else {
            Logger::get_instance().log(LogLevel::Info, "Write failed.", None);
        }
    }
}
