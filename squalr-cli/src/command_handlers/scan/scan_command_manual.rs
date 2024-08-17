use crate::command_handlers::scan::ScanCommand;
use crate::session_manager::SessionManager;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_scanning::scanners::constraints::scan_constraint::ScanConstraint;
use squalr_engine_scanning::scanners::manual_scanner::ManualScanner;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use squalr_engine_scanning::snapshots::snapshot_manager::SnapshotManager;
use std::thread;

pub fn handle_manual_scan_command(cmd: &mut ScanCommand) {
    let session_manager_lock = SessionManager::get_instance();
    let session_manager = session_manager_lock.read().unwrap();
    let snapshot_manager_lock = SnapshotManager::get_instance();
    let mut snapshot_manager = snapshot_manager_lock.write().unwrap();

    if let ScanCommand::Manual { value_and_type, constraint_type} = cmd {
        if let Some(process_info) = session_manager.get_opened_process() {

            // First collect values before the new scan
            let snapshot = snapshot_manager.get_active_snapshot_create_if_none(&process_info);
            ValueCollector::collect_values(
                process_info.clone(),
                snapshot,
                None,
                true,
            ).wait_for_completion();
            
            // Now set up for the memory scan
            let constraint = ScanConstraint::new_with_value(
                MemoryAlignment::Alignment1,
                constraint_type.to_owned(),
                value_and_type.data_type.to_owned(),
                Some(value_and_type.data_value.to_owned()));
            let snapshot = snapshot_manager.get_active_snapshot_create_if_none(&process_info);
            
            let task = ManualScanner::scan(
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
