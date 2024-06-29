pub mod scan_command;
pub use scan_command::ScanCommand;
use squalr_engine_common::logging::logger::LOGGER;
use squalr_engine_common::logging::log_level::LogLevel;

pub fn handle_scan_command(cmd: ScanCommand) {
    match cmd {
        ScanCommand::Value { value } => {
            LOGGER.log(LogLevel::Info, &format!("Scanning for value: {}", value), None);
        }
    }
}
