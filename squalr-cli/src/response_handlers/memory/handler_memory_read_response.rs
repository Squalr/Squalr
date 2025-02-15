use squalr_engine::commands::memory::responses::memory_response::MemoryResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_memory_read_response(memory_response: MemoryResponse) {
    // Logger::get_instance().log(LogLevel::Info, &format!("Reading value from address {}", address), None);
    if let MemoryResponse::Read { value, address, success } = memory_response {
        if success {
            Logger::get_instance().log(LogLevel::Info, &format!("Read value {:?} from address {}", value, address), None);
        } else {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to read memory"), None);
        }
    }
}
