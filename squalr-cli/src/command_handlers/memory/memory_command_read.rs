use crate::command_handlers::memory::memory_command::MemoryCommand;
use crate::session_manager::SessionManager;

use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;

pub fn handle_memory_read(cmd: MemoryCommand) {
    if let MemoryCommand::Read { address, mut value } = cmd {
        let session_manager_lock = SessionManager::instance();
        let session_manager = session_manager_lock.read().unwrap();
        if let Some(process_info) = session_manager.get_opened_process() {
            Logger::instance().log(
                LogLevel::Info,
                &format!("Reading value from address {}", address),
                None
            );
            
            match MemoryReader::instance().read(process_info.handle, address, &mut value) {
                Ok(_) => {
                    Logger::instance().log(
                        LogLevel::Info,
                        &format!("Read value {:?} from address {}", value, address),
                        None
                    );
                }
                Err(e) => {
                    Logger::instance().log(
                        LogLevel::Error,
                        &format!("Failed to read memory: {}", e),
                        None
                    );
                }
            }
        } else {
            Logger::instance().log(
                LogLevel::Error,
                "No opened process available.",
                None
            );
        }
    }
}
