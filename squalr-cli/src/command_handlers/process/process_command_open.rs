use crate::command_handlers::process::process_command::ProcessCommand;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_processes::process_query::process_queryer::ProcessQueryOptions;
use sysinfo::Pid;

pub fn handle_process_open(cmd: &mut ProcessCommand) {
    let session_manager_lock = SessionManager::get_instance();
    let mut session_manager = session_manager_lock.write().unwrap();

    if let ProcessCommand::Open { pid, search_name, match_case } = cmd {
        if pid.is_none() && search_name.is_none() {
            Logger::get_instance().log(LogLevel::Error, "Error: Neither PID nor search name provided. Cannot open process.", None);
            return;
        }

        Logger::get_instance().log(LogLevel::Info, "Opening process", None);

        let mut queryer = ProcessQuery::;
        let options = ProcessQueryOptions {
            require_windowed: false,
            required_pid: pid.map(Pid::from_u32),
            search_name: search_name.clone(),
            match_case: *match_case,
            limit: Some(1),
        };

        let processes = queryer.get_processes(options);

        if let Some(process_info) = processes.first() {
            let queryer = ProcessQuery::;

            match queryer.open_process(&process_info) {
                Ok(opened_process_info) => {
                    session_manager.set_opened_process(opened_process_info.clone());
                    Logger::get_instance().log(LogLevel::Info, &format!("Process opened: {:?}", opened_process_info.pid), None);
                }
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to open process {}: {}", process_info.pid, err), None);
                }
            }
        } else {
            Logger::get_instance().log(LogLevel::Warn, "No matching process found.", None);
        }
    }
}
