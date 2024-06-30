use crate::command_handlers::process::process_command::ProcessCommand;
use crate::session_manager::SESSION_MANAGER;
use squalr_engine_common::logging::logger::LOGGER;
use squalr_engine_common::logging::log_level::LogLevel;

pub fn handle_process_close(_cmd: ProcessCommand) {
    let mut session_manager = SESSION_MANAGER.lock().unwrap();
    if let Some(opened_pid) = session_manager.get_opened_process() {
        LOGGER.log(LogLevel::Info, &format!("Closing process {}", opened_pid.as_u32()), None);
        session_manager.set_opened_process(None);
        LOGGER.log(LogLevel::Info, "Process closed", None);
    } else {
        LOGGER.log(LogLevel::Info, "No process to close", None);
    }
}
