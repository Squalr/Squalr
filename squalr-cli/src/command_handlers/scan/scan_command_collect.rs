use crate::command_handlers::scan::ScanCommand;
use crate::session_manager::SessionManager;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use squalr_engine_scanning::snapshots::snapshot_manager::SnapshotManager;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;

pub fn handle_collect_command(cmd: ScanCommand) {
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

            task.on_progress_updated(|progress| {
                Logger::instance().log(LogLevel::Info, &format!("Progress: {:.2}%", progress), None);
            });

            task.wait_for_completion();
        } else {
            Logger::instance().log(LogLevel::Info, "No opened process", None);
        }
    }
}
