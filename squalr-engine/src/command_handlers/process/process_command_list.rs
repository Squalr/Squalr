use crate::commands::process::process_command::ProcessCommand;
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

        let options = ProcessQueryOptions {
            search_name: search_name.as_ref().cloned(),
            required_pid: None,
            require_windowed: *require_windowed,
            match_case: *match_case,
            fetch_icons: false,
            limit: *limit,
        };

        let processes = ProcessQuery::get_processes(options);

        for process_info in processes {
            Logger::get_instance().log(LogLevel::Info, &format!("PID: {}, Name: {}", process_info.pid, process_info.name), None);
        }
    }
}
