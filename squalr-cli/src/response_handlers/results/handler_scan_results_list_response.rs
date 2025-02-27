use squalr_engine::command_executors::scan_results::list::scan_results_list_response::ScanResultsListResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_scan_new_response(results_list_response: ScanResultsListResponse) {
    for result_index in initial_index..end_index {
        if let Some(scan_result) = snapshot.get_scan_result(result_index, &self.data_type) {
            Logger::log(LogLevel::Info, format!("{:?}", scan_result).as_str(), None);
        }
    }
}
