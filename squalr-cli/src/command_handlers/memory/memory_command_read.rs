use crate::command_handlers::memory::memory_command::MemoryCommand;
use squalr_engine_common::logging::logger::LOGGER;
use squalr_engine_common::logging::log_level::LogLevel;

pub fn handle_memory_read(cmd: MemoryCommand) {
    if let MemoryCommand::Read { address, value } = cmd {
        LOGGER.log(
            LogLevel::Info,
            &format!(
                "Reading value from address {}",
                address
            ),
            None
        );
    }
}
