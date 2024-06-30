use crate::command_handlers::memory::memory_command::MemoryCommand;
use squalr_engine_common::logging::logger::LOGGER;
use squalr_engine_common::logging::log_level::LogLevel;

pub fn handle_memory_write(cmd: MemoryCommand) {
    if let MemoryCommand::Write { address, value } = cmd {
        LOGGER.log(
            LogLevel::Info,
            &format!(
                "Writing value {:?} to address {}",
                value, address
            ),
            None
        );
    }
}
