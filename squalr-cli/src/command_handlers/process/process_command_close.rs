use crate::command_handlers::process::process_command::ProcessCommand;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;

pub fn handle_process_close(cmd: &mut ProcessCommand) {
    if let ProcessCommand::Close {} = cmd {
        let session_manager_lock = SessionManager::get_instance();
        let mut session_manager = session_manager_lock.write().unwrap();

        if let Some(process_info) = session_manager.get_opened_process() {
            Logger::get_instance().log(
                LogLevel::Info,
                &format!("Closing process {} with handle {}", process_info.pid, process_info.handle),
                None,
            );

            match ProcessQuery::close_process(process_info.handle) {
                Ok(_) => {
                    session_manager.clear_opened_process();
                }
                Err(e) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to close process handle {}: {}", process_info.handle, e), None);
                }
            }
        } else {
            Logger::get_instance().log(LogLevel::Info, "No process to close", None);
        }
    }
}
