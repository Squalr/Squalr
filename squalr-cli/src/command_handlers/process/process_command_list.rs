use crate::command_handlers::process::process_command::ProcessCommand;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_processes::process_query::process_queryer::ProcessQueryOptions;

pub fn handle_process_list(cmd: &mut ProcessCommand) {
    if let ProcessCommand::List {
        require_windowed,
        search_name,
        match_case,
        limit,
    } = cmd
    {
        Logger::get_instance().log(
            LogLevel::Info,
            &format!(
                "Listing processes with options: require_windowed={}, search_name={:?}, match_case={}, limit={:?}",
                require_windowed, search_name, match_case, limit
            ),
            None,
        );

        let mut queryer = ProcessQuery::get_instance();
        let options = ProcessQueryOptions {
            require_windowed: *require_windowed,
            search_name: search_name.as_ref().cloned(),
            match_case: *match_case,
            limit: *limit,
        };

        let processes = queryer.get_processes(options);

        for pid in processes {
            if let Some(name) = queryer.get_process_name(pid) {
                Logger::get_instance().log(LogLevel::Info, &format!("PID: {}, Name: {}", pid, name), None);
            }
        }
    }
}
