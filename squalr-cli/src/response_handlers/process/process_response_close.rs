use squalr_engine::responses::process::process_response::ProcessResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_process_response_close(process_response: ProcessResponse) {
    if let ProcessResponse::Open { process_info } = process_response {
        Logger::get_instance().log(LogLevel::Info, &format!("Closed pid: {}, name: {}", process_info.pid, process_info.name), None);
    }
}
