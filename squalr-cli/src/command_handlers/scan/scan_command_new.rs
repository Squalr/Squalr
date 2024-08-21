use crate::command_handlers::scan::ScanCommand;
use squalr_engine::session_manager::SessionManager;

pub fn handle_new_scan_command(
    cmd: &mut ScanCommand,
) {
    if let ScanCommand::New { scan_filter_parameters } = cmd {
        let session_manager_lock = SessionManager::get_instance();
        let process_info = {
            let session_manager = session_manager_lock.read().unwrap();
            session_manager.get_opened_process().cloned()
        };

        if let Some(process_info) = process_info {
            let session_manager = session_manager_lock.write().unwrap();
            let snapshot = session_manager.get_snapshot();
            let mut snapshot = snapshot.write().unwrap();

            snapshot.new_scan(&process_info, scan_filter_parameters.clone());
        }
    }
}
