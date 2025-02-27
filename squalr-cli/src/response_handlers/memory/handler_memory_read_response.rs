use squalr_engine::command_executors::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_memory_read_response(memory_read_response: MemoryReadResponse) {
    // Logger::log(LogLevel::Info, &format!("Reading value from address {}", address), None);

    if memory_read_response.success {
        Logger::log(
            LogLevel::Info,
            &format!("Read value {:?} from address {}", memory_read_response.value, memory_read_response.address),
            None,
        );
    } else {
        Logger::log(LogLevel::Error, &format!("Failed to read memory"), None);
    }
}
