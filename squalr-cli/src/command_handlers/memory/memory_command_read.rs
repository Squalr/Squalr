use crate::command_handlers::memory::memory_command::MemoryCommand;
use crate::session_manager::SESSION_MANAGER;

use squalr_engine_common::logging::logger::LOGGER;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_memory::memory_reader::MemoryReader;

pub fn handle_memory_read(cmd: MemoryCommand) {
    if let MemoryCommand::Read { address, mut value } = cmd {
        let session_manager = SESSION_MANAGER.lock().unwrap();
        if let Some(opened_pid) = session_manager.get_opened_process() {
            LOGGER.log(
                LogLevel::Info,
                &format!("Reading value from address {}", address),
                None
            );

            let memory_reader = MemoryReader::instance();
            let memory_reader = memory_reader.lock().expect("Failed to acquire memory reader lock");

            match memory_reader.read(&opened_pid, address, &mut value) {
                Ok(_) => {
                    LOGGER.log(
                        LogLevel::Info,
                        &format!("Read value {:?} from address {}", value, address),
                        None
                    );
                }
                Err(e) => {
                    LOGGER.log(
                        LogLevel::Error,
                        &format!("Failed to read memory: {}", e),
                        None
                    );
                }
            }
        } else {
            LOGGER.log(
                LogLevel::Error,
                "No opened process available.",
                None
            );
        }
    }
}
