pub mod scan_command;
pub mod scan_command_manual;
pub mod scan_command_value_collector;

pub use scan_command::ScanCommand;
pub use scan_command_manual::handle_manual_scan_command;
pub use scan_command_value_collector::handle_value_collector_command;

pub async fn handle_scan_command(cmd: &mut ScanCommand) {
    match cmd {
        ScanCommand::Value { .. } => handle_manual_scan_command(cmd).await,
        ScanCommand::Collect => handle_value_collector_command(cmd).await,
    }
}
