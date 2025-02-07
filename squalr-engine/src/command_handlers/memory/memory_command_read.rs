use crate::commands::memory::memory_command::MemoryCommand;
use crate::squalr_session::SqualrSession;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;

pub fn handle_memory_read(cmd: MemoryCommand) {
    if let MemoryCommand::Read { address, value } = cmd {
        if let Some(process_info) = SqualrSession::get_opened_process() {
            Logger::get_instance().log(LogLevel::Info, &format!("Reading value from address {}", address), None);

            let mut out_value = value.clone();

            match MemoryReader::get_instance().read(&process_info, address, &mut out_value) {
                true => {
                    Logger::get_instance().log(LogLevel::Info, &format!("Read value {:?} from address {}", out_value, address), None);
                }
                false => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to read memory"), None);
                }
            }
        } else {
            Logger::get_instance().log(LogLevel::Error, "No opened process available.", None);
        }
    }
}
