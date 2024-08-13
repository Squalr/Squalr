use crate::command_handlers::scan::ScanCommand;

use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;

pub async fn handle_value_command(cmd: &mut ScanCommand) {
    if let ScanCommand::Value { value } = cmd {
        Logger::get_instance().log(LogLevel::Info, &format!("Scanning for value: {}", value), None);
    }
}
