use crate::command_handlers::scan::ScanCommand;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use std::thread;

pub fn handle_value_collector_command(cmd: &mut ScanCommand) {
    let session_manager_lock = SessionManager::get_instance();
    let session_manager = session_manager_lock.read().unwrap();

    if let ScanCommand::Collect = cmd {
        if let Some(process_info) = session_manager.get_opened_process() {

            let task = ValueCollector::collect_values(
                process_info.clone(),
                session_manager.get_scan_results(),
                None,
                true,
            );

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
