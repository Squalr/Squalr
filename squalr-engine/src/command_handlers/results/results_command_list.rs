use crate::command_handlers::results::results_command::ResultsCommand;
use crate::squalr_engine::SqualrEngine;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_scanning::scan_settings::ScanSettings;

pub fn handle_results_list(cmd: &mut ResultsCommand) {
    // Irrefutable pattern, so no `let =` as is present in most other command handlers.
    let ResultsCommand::List { page, data_type } = cmd;
    {
        let results_page_size = ScanSettings::get_instance().get_results_page_size() as u64;
        let initial_index = *page * results_page_size;
        let end_index = initial_index + results_page_size;
        let snapshot_lock = SqualrEngine::get_snapshot();
        let snapshot = snapshot_lock.read().unwrap();

        for result_index in initial_index..end_index {
            if let Some(scan_result) = snapshot.get_scan_result(result_index, &data_type) {
                Logger::get_instance().log(LogLevel::Info, format!("{:?}", scan_result).as_str(), None);
            }
        }
    }
}
