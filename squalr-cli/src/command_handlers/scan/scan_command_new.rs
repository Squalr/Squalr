use crate::command_handlers::scan::ScanCommand;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_scanning::scanners::constraints::scan_constraint::ScanConstraint;
use squalr_engine_scanning::scanners::manual_scanner::ManualScanner;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use std::thread;

pub fn handle_manual_scan_command(
    cmd: &mut ScanCommand,
) {
    if let ScanCommand::New { filter_constraints } = cmd {
        let session_manager_lock = SessionManager::get_instance();
        let mut session_manager = session_manager_lock.write().unwrap();
        
        // TODO
    }
}
