use crate::command_handlers::scan::ScanCommand;

use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;

pub fn handle_value_command(cmd: ScanCommand) {
    if let ScanCommand::Value { value } = cmd {
        Logger::instance().log(LogLevel::Info, &format!("Scanning for value: {}", value), None);
    }
}
