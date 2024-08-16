pub mod scan_command;
pub mod scan_command_hybrid;
pub mod scan_command_manual;
pub mod scan_command_value_collector;

pub use scan_command::ScanCommand;
pub use scan_command_hybrid::handle_hybrid_scan_command;
pub use scan_command_manual::handle_manual_scan_command;
pub use scan_command_value_collector::handle_value_collector_command;

pub fn handle_scan_command(cmd: &mut ScanCommand) {
    match cmd {
        ScanCommand::Manual { .. } => handle_manual_scan_command(cmd),
        ScanCommand::Hybrid { .. } => handle_hybrid_scan_command(cmd),
        ScanCommand::Collect => handle_value_collector_command(cmd),
    }
}
