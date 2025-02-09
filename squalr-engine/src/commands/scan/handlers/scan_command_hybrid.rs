use crate::squalr_session::SqualrSession;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::values::anonymous_value::AnonymousValue;
use squalr_engine_scanning::scanners::hybrid_scanner::HybridScanner;
use squalr_engine_scanning::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_scanning::scanners::parameters::scan_parameters::ScanParameters;
use std::thread;
use uuid::Uuid;

pub fn handle_hybrid_scan_command(
    scan_value: &Option<AnonymousValue>,
    compare_type: &ScanCompareType,
    uuid: Uuid,
) {
    if let Some(process_info) = SqualrSession::get_opened_process() {
        let snapshot = SqualrSession::get_snapshot();
        let scan_parameters = ScanParameters::new_with_value(compare_type.to_owned(), scan_value.to_owned());

        // Perform the hybrid scan which simultaneously collects and scans memory.
        let task = HybridScanner::scan(process_info.clone(), snapshot, &scan_parameters, None, true);

        // Spawn a thread to listen to progress updates
        let progress_receiver = task.add_listener();
        thread::spawn(move || {
            while let Ok(progress) = progress_receiver.recv() {
                Logger::get_instance().log(LogLevel::Info, &format!("Progress: {:.2}%", progress), None);
            }
        });

        // Wait for completion synchronously
        task.wait_for_completion();
    } else {
        Logger::get_instance().log(LogLevel::Info, "No opened process", None);
    }
}
