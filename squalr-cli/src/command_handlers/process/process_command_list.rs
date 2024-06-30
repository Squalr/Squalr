use crate::command_handlers::process::process_command::ProcessCommand;
use squalr_engine_common::logging::logger::LOGGER;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_processes::process_query::ProcessQuery;

pub fn handle_process_list(cmd: ProcessCommand) {
    if let ProcessCommand::List { windowed, search_term, match_case, system_processes, limit } = cmd {
        LOGGER.log(
            LogLevel::Info,
            &format!(
                "Listing processes with options: windowed={}, search_term={:?}, match_case={}, system_processes={}, limit={:?}",
                windowed, search_term, match_case, system_processes, limit
            ),
            None
        );

        let mut queryer = ProcessQuery::instance();
        let mut processes = queryer.get_processes();

        if let Some(limit) = limit {
            processes.truncate(limit as usize);
        }

        let filtered_processes: Vec<_> = processes.into_iter()
            .filter(|pid| {
                let is_system = queryer.is_process_system_process(pid);
                if system_processes || !is_system {
                    if let Some(name) = queryer.get_process_name(*pid) {
                        let mut matches = true;
                        if windowed {
                            matches &= queryer.is_process_windowed(pid);
                        }
                        if let Some(ref term) = search_term {
                            if match_case {
                                matches &= name.contains(term);
                            } else {
                                matches &= name.to_lowercase().contains(&term.to_lowercase());
                            }
                        }
                        matches
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect();

        for pid in filtered_processes {
            if let Some(name) = queryer.get_process_name(pid) {
                LOGGER.log(
                    LogLevel::Info,
                    &format!("PID: {}, Name: {}", pid, name),
                    None
                );
            }
        }
    }
}
