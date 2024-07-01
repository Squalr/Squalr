use crate::command_handlers::process::process_command::ProcessCommand;
use crate::session_manager::SessionManager;

use squalr_engine_processes::process_query::ProcessQuery;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;

pub async fn handle_process_close(_cmd: &mut ProcessCommand) {
    let session_manager_lock = SessionManager::instance();
    let mut session_manager = session_manager_lock.write().unwrap();

    if let Some(process_info) = session_manager.get_opened_process() {
        Logger::instance().log(
            LogLevel::Info,
            &format!("Closing process {} with handle {}", process_info.pid, process_info.handle),
            None,
        );

        let queryer = ProcessQuery::instance();

        match queryer.close_process(process_info.handle) {
            Ok(_) => {
                session_manager.clear_opened_process();
                Logger::instance().log(LogLevel::Info, "Process closed", None);
            }
            Err(e) => {
                Logger::instance().log(
                    LogLevel::Error,
                    &format!("Failed to close process handle {}: {}", process_info.handle, e),
                    None,
                );
            }
        }
    } else {
        Logger::instance().log(LogLevel::Info, "No process to close", None);
    }
}
