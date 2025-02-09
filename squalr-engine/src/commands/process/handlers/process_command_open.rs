use crate::responses::engine_response::EngineResponse;
use crate::responses::process::process_response::ProcessResponse;
use crate::squalr_engine::SqualrEngine;
use crate::squalr_session::SqualrSession;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_processes::process_query::process_queryer::ProcessQueryOptions;
use sysinfo::Pid;
use uuid::Uuid;

pub fn handle_process_open(
    pid: Option<u32>,
    search_name: &Option<String>,
    match_case: bool,
    uuid: Uuid,
) {
    if pid.is_none() && search_name.is_none() {
        Logger::get_instance().log(LogLevel::Error, "Error: Neither PID nor search name provided. Cannot open process.", None);
        return;
    }

    Logger::get_instance().log(LogLevel::Info, "Opening process", None);

    let options = ProcessQueryOptions {
        search_name: search_name.clone(),
        required_pid: pid.map(Pid::from_u32),
        require_windowed: false,
        match_case: match_case,
        fetch_icons: false,
        limit: Some(1),
    };

    let processes = ProcessQuery::get_processes(options);

    if let Some(process_info) = processes.first() {
        match ProcessQuery::open_process(&process_info) {
            Ok(opened_process_info) => {
                SqualrSession::set_opened_process(opened_process_info.clone());

                let response = EngineResponse::Process(ProcessResponse::Close {
                    process_info: opened_process_info,
                });

                SqualrEngine::dispatch_response(response, uuid);
            }
            Err(err) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to open process {}: {}", process_info.pid, err), None);
            }
        }
    } else {
        Logger::get_instance().log(LogLevel::Warn, "No matching process found.", None);
    }
}
