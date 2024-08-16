use crate::command_handlers::scan::ScanCommand;
use crate::session_manager::SessionManager;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_scanning::scanners::constraints::scan_constraint::ScanConstraint;
use squalr_engine_scanning::scanners::manual_scanner::ManualScanner;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use squalr_engine_scanning::snapshots::snapshot_manager::SnapshotManager;
use tokio::spawn;

pub async fn handle_manual_scan_command(cmd: &mut ScanCommand) {
    let session_manager_lock = SessionManager::get_instance();
    let session_manager = session_manager_lock.read().unwrap();
    let snapshot_manager_lock = SnapshotManager::get_instance();
    let mut snapshot_manager = snapshot_manager_lock.write().unwrap();

    if let ScanCommand::Value { value, constraint_type, delta_value } = cmd {
        if let Some(process_info) = session_manager.get_opened_process() {

            // First collect values before the new scan
            let snapshot = snapshot_manager.get_active_snapshot_create_if_none(&process_info);
            ValueCollector::collect_values(
                process_info.clone(),
                snapshot,
                None,
                true,
            ).wait_for_completion().await;
            
            // Now set up for the memory scan
            let constraint = ScanConstraint::new_with_value(MemoryAlignment::Alignment1, constraint_type.to_owned(), value.to_owned(), delta_value.to_owned());
            let snapshot = snapshot_manager.get_active_snapshot_create_if_none(&process_info);
            let task = ManualScanner::scan(
                snapshot,
                &constraint,
                None,
                true
            );

            // Subscribe to progress updates
            let mut progress_receiver = task.get_progress_receiver();
            spawn(async move {
                while let Ok(progress) = progress_receiver.recv().await {
                    Logger::get_instance().log(LogLevel::Info, &format!("Progress: {:.2}%", progress), None);
                }
            });

            // Wait for completion
            task.wait_for_completion().await;
        } else {
            Logger::get_instance().log(LogLevel::Info, "No opened process", None);
        }
    }
}
