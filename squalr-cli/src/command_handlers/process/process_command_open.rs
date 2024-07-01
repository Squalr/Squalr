use crate::command_handlers::process::process_command::ProcessCommand;
use crate::session_manager::SessionManager;

use squalr_engine_processes::process_query::ProcessQuery;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use sysinfo::Pid;

pub fn handle_process_open(cmd: ProcessCommand) {
    let session_manager_lock = SessionManager::instance();
    let mut session_manager = session_manager_lock.write().unwrap();

    if let ProcessCommand::Open { pid } = cmd {
        Logger::instance().log(LogLevel::Info, "Opening process", None);

        let pid = Pid::from_u32(pid);
        let queryer = ProcessQuery::instance();

        match queryer.open_process(&pid) {
            Ok(handle) => {
                session_manager.set_opened_process(pid, handle);
                Logger::instance().log(
                    LogLevel::Info,
                    &format!("Process {} opened with handle {}", pid, handle),
                    None,
                );
            },
            Err(e) => {
                Logger::instance().log(
                    LogLevel::Error,
                    &format!("Failed to open process {}: {}", pid, e),
                    None,
                );
            },
        }
    }
}
