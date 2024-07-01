use crate::command_handlers::process::process_command::ProcessCommand;
use crate::session_manager::SessionManager;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;

pub fn handle_process_close(_cmd: ProcessCommand) {
    let session_manager_lock = SessionManager::instance();
    let mut session_manager = session_manager_lock.write().unwrap();
    if let Some(opened_pid) = session_manager.get_opened_process() {
        Logger::instance().log(LogLevel::Info, &format!("Closing process {}", opened_pid.as_u32()), None);
        session_manager.set_opened_process(None);
        Logger::instance().log(LogLevel::Info, "Process closed", None);
    } else {
        Logger::instance().log(LogLevel::Info, "No process to close", None);
    }
}
