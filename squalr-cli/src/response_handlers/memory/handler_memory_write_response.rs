use squalr_engine::command_executors::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_memory_response_write(memory_response: MemoryWriteResponse) {
    if memory_response.success {
        Logger::log(LogLevel::Info, "Write success.", None);
    } else {
        Logger::log(LogLevel::Info, "Write failed.", None);
    }
}
