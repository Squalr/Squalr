use crate::command_handlers::scan::ScanCommand;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_scanning::scanners::constraints::scan_constraint::ScanConstraint;
use squalr_engine_scanning::scanners::hybrid_scanner::HybridScanner;
use std::thread;

pub fn handle_hybrid_scan_command(
    cmd: &mut ScanCommand,
) {
    if let ScanCommand::Hybrid { value_and_type, constraint_type} = cmd {
        if let Some(process_info) = SessionManager::get_instance().read().unwrap().get_opened_process() {
            let snapshot = SessionManager::get_instance().write().unwrap().get_or_create_snapshot(process_info);

            let data_types = vec![value_and_type.data_type.to_owned()];

            let constraint = ScanConstraint::new_with_value(
                MemoryAlignment::Alignment1,
                constraint_type.to_owned(),
                data_types,
                Some(value_and_type.data_value.to_owned()));
            
            let task = HybridScanner::scan(
                process_info.clone(),
                snapshot,
                &constraint,
                None,
                true
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
