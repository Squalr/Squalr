use crate::command_handlers::process::process_command::ProcessCommand;
use crate::session_manager::SessionManager;
use squalr_engine_processes::process_query::ProcessQuery;
use squalr_engine_processes::process_query::ProcessQueryOptions;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use sysinfo::Pid;

pub async fn handle_process_open(cmd: &mut ProcessCommand) {
    let session_manager_lock = SessionManager::get_instance();
    let mut session_manager = session_manager_lock.write().unwrap();

    if let ProcessCommand::Open {pid, search_name, match_case, include_system_processes } = cmd {
        if pid.is_none() && search_name.is_none() {
            Logger::get_instance().log(
                LogLevel::Error,
                "Error: Neither PID nor search name provided. Cannot open process.",
                None,
            );
            return;
        }

        Logger::get_instance().log(LogLevel::Info, "Opening process", None);

        let mut pid = match pid {
            Some(pid) => Pid::from_u32(*pid),
            None => Pid::from_u32(0),
        };

        // Overwrite pid if a search name is provided
        if search_name.is_some() {
            let mut queryer = ProcessQuery::get_instance();
            let options = ProcessQueryOptions {
                require_windowed: false,
                search_name: search_name.as_ref().cloned(),
                match_case: *match_case,
                include_system_processes: *include_system_processes,
                limit: Some(1),
            };
    
            let processes = queryer.get_processes(options);

            if let Some(found_pid) = processes.first() {
                pid = *found_pid;
            }
        }

        let queryer = ProcessQuery::get_instance();

        match queryer.open_process(&pid) {
            Ok(process_info) => {
                session_manager.set_opened_process(process_info);
                Logger::get_instance().log(
                    LogLevel::Info,
                    &format!("Process opened: {:?}", process_info),
                    None,
                );
            },
            Err(e) => {
                Logger::get_instance().log(
                    LogLevel::Error,
                    &format!("Failed to open process {}: {}", pid, e),
                    None,
                );
            },
        }
    }
}
