use crate::command_handlers::process::process_command::ProcessCommand;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_processes::process_query::{ProcessQuery, ProcessQueryOptions};

pub fn handle_process_list(cmd: ProcessCommand) {
    if let ProcessCommand::List { windowed, search_term, match_case, system_processes, limit } = cmd {
        Logger::instance().log(
            LogLevel::Info,
            &format!(
                "Listing processes with options: windowed={}, search_term={:?}, match_case={}, system_processes={}, limit={:?}",
                windowed, search_term, match_case, system_processes, limit
            ),
            None
        );

        let mut queryer = ProcessQuery::instance();
        let options = ProcessQueryOptions {
            windowed,
            search_term,
            match_case,
            system_processes,
            limit,
        };

        let processes = queryer.get_processes(options);

        for pid in processes {
            if let Some(name) = queryer.get_process_name(pid) {
                Logger::instance().log(
                    LogLevel::Info,
                    &format!("PID: {}, Name: {}", pid, name),
                    None
                );
            }
        }
    }
}
