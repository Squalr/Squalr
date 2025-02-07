use crate::command_handlers::scan::ScanCommand;
use crate::squalr_session::SqualrSession;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use std::thread;

pub fn handle_value_collector_command(cmd: ScanCommand) {
    if let ScanCommand::Collect = cmd {
        if let Some(process_info) = SqualrSession::get_opened_process() {
            let snapshot = SqualrSession::get_snapshot();
            let task = ValueCollector::collect_values(process_info.clone(), snapshot, None, true);

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
}
