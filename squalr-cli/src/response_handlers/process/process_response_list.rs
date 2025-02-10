use squalr_engine::responses::process::process_response::ProcessResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_process_response_list(process_response: ProcessResponse) {
    if let ProcessResponse::List { processes } = process_response {
        for process_info in processes {
            Logger::get_instance().log(
                LogLevel::Info,
                &format!("process_id: {}, name: {}", process_info.process_id, process_info.name),
                None,
            );
        }
    }
}
