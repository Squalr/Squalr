pub mod scan_command;

pub use scan_command::ScanCommand;

pub fn handle_scan_command(cmd: ScanCommand) {
    match cmd {
        ScanCommand::Value { value } => {
            println!("Scanning for value: {}", value);
        }
    }
}
