use crate::squalr_session::SqualrSession;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_scanning::scan_settings::ScanSettings;
use uuid::Uuid;

pub fn handle_results_list(
    page: u64,
    data_type: &DataType,
    uuid: Uuid,
) {
    let results_page_size = ScanSettings::get_instance().get_results_page_size() as u64;
    let initial_index = page * results_page_size;
    let end_index = initial_index + results_page_size;
    let snapshot_lock = SqualrSession::get_snapshot();
    let snapshot = snapshot_lock.read().unwrap();

    for result_index in initial_index..end_index {
        if let Some(scan_result) = snapshot.get_scan_result(result_index, &data_type) {
            Logger::get_instance().log(LogLevel::Info, format!("{:?}", scan_result).as_str(), None);
        }
    }
}
