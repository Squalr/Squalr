use crate::command_handlers::scan::ScanCommand;
use crate::session_manager::SessionManager;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use squalr_engine_scanning::snapshots::snapshot_manager::SnapshotManager;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use tokio::spawn;

pub async fn handle_collect_command(cmd: &mut ScanCommand) {
    let session_manager_lock = SessionManager::instance();
    let session_manager = session_manager_lock.read().unwrap();

    let snapshot_manager_lock = SnapshotManager::instance();
    let mut snapshot_manager = snapshot_manager_lock.write().unwrap();

    if let ScanCommand::Collect = cmd {
        if let Some(process_info) = session_manager.get_opened_process() {
            Logger::instance().log(LogLevel::Info, "Collecting values", None);

            let snapshot = snapshot_manager.get_active_snapshot_create_if_none(&process_info.pid);
            let task = ValueCollector::collect_values(
                process_info.clone(),
                snapshot,
                None,
                true,
            );

            // Subscribe to progress updates
            let mut progress_receiver = task.progress_receiver();
            spawn(async move {
                while let Ok(progress) = progress_receiver.recv().await {
                    Logger::instance().log(LogLevel::Info, &format!("Progress: {:.2}%", progress), None);
                }
            });

            // Wait for completion
            task.wait_for_completion().await;
        } else {
            Logger::instance().log(LogLevel::Info, "No opened process", None);
        }
    }
}
