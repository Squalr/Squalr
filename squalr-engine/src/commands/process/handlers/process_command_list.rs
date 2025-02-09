use crate::responses::engine_response::EngineResponse;
use crate::responses::process::process_response::ProcessResponse;
use crate::squalr_engine::SqualrEngine;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_processes::process_query::process_queryer::ProcessQueryOptions;
use uuid::Uuid;

pub fn handle_process_command_list(
    require_windowed: bool,
    search_name: &Option<String>,
    match_case: bool,
    limit: Option<u64>,
    fetch_icons: bool,
    uuid: Uuid,
) {
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
        require_windowed: require_windowed,
        match_case: match_case,
        fetch_icons: fetch_icons,
        limit: limit,
    };

    let processes = ProcessQuery::get_processes(options);
    let response = EngineResponse::Process(ProcessResponse::List { processes: processes });

    SqualrEngine::dispatch_response(response, uuid);
}
