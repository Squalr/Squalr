use squalr_engine::commands::process::process_response::ProcessResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_process_listen_response(process_response: ProcessResponse) {
    if let ProcessResponse::Listen { process_listen_response } = process_response {
        let process_info = process_listen_response.opened_process_info;

        if let Some(process_info) = process_info {
            Logger::get_instance().log(
                LogLevel::Info,
                &format!("Opened process process_id: {}, Name: {}", process_info.process_id, process_info.name),
                None,
            );
        } else {
            Logger::get_instance().log(LogLevel::Info, "Attached process closed.", None);
        }
    }
}
