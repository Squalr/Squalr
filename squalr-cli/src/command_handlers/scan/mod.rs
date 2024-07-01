pub mod scan_command;
pub mod scan_command_value;
pub mod scan_command_collect;

pub use scan_command::ScanCommand;
pub use scan_command_value::handle_value_command;
pub use scan_command_collect::handle_collect_command;

pub async fn handle_scan_command(cmd: &mut ScanCommand) {
    match cmd {
        ScanCommand::Value { .. } => handle_value_command(cmd).await,
        ScanCommand::Collect => handle_collect_command(cmd).await,
    }
}
