pub mod scan_command_hybrid;
pub mod scan_command_manual;
pub mod scan_command_new;
pub mod scan_command_value_collector;

use crate::command_handlers::scan::scan_command_hybrid::handle_hybrid_scan_command;
use crate::command_handlers::scan::scan_command_manual::handle_manual_scan_command;
use crate::command_handlers::scan::scan_command_new::handle_new_scan_command;
use crate::command_handlers::scan::scan_command_value_collector::handle_value_collector_command;
use crate::commands::scan::scan_command::ScanCommand;

pub fn handle_scan_command(cmd: ScanCommand) {
    match cmd {
        ScanCommand::Collect => handle_value_collector_command(cmd),
        ScanCommand::Hybrid { .. } => handle_hybrid_scan_command(cmd),
        ScanCommand::Manual { .. } => handle_manual_scan_command(cmd),
        ScanCommand::New { .. } => handle_new_scan_command(cmd),
    }
}
