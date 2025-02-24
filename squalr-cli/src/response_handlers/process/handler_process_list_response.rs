use squalr_engine::commands::process::process_response::ProcessResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_process_list_response(process_response: ProcessResponse) {
    if let ProcessResponse::List { process_list_response } = process_response {
        let processes = process_list_response.processes;

        for process_info in processes {
            Logger::log(
                LogLevel::Info,
                &format!("process_id: {}, name: {}", process_info.process_id, process_info.name),
                None,
            );
        }
    }
}
