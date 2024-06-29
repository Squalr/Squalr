use crate::command_handlers::process::process_command::ProcessCommand;
use crate::session_manager::SESSION_MANAGER;
use squalr_engine_common::logging::logger::LOGGER;
use squalr_engine_common::logging::log_level::LogLevel;
use sysinfo::Pid;

pub fn handle_process_open(cmd: ProcessCommand) {
    if let ProcessCommand::Open { pid } = cmd {
        LOGGER.log(LogLevel::Info, "Opening process", None);

        let pid = Pid::from_u32(pid);
        let mut session_manager = SESSION_MANAGER.lock().unwrap();
        session_manager.set_opened_process(Some(pid));

        LOGGER.log(LogLevel::Info, &format!("Process {} opened", pid), None);
    }
}
