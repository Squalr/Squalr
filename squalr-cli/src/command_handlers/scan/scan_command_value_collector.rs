use crate::command_handlers::scan::ScanCommand;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use std::thread;

pub fn handle_value_collector_command(
    cmd: &mut ScanCommand,
) {
    if let ScanCommand::Collect = cmd {
        let session_manager_lock = SessionManager::get_instance();
        let process_info = {
            let session_manager = session_manager_lock.read().unwrap();
            session_manager.get_opened_process().cloned()
        };

        if let Some(process_info) = process_info {
            let mut session_manager = session_manager_lock.write().unwrap();
            let snapshot = session_manager.get_or_create_snapshot(&process_info);
            let task = ValueCollector::collect_values(
                process_info.clone(),
                snapshot,
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
