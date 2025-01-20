use crate::command_handlers::memory::memory_command::MemoryCommand;
use crate::session_manager::SessionManager;
use squalr_engine_common::dynamic_struct::to_bytes::ToBytes;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_memory::memory_writer::MemoryWriter;
use squalr_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;

pub fn handle_memory_write(cmd: &mut MemoryCommand) {
    if let MemoryCommand::Write { address, value } = cmd {
        let session_manager_lock = SessionManager::get_instance();
        let session_manager = session_manager_lock.read().unwrap();

        if let Some(process_info) = session_manager.get_opened_process() {
            // Log the memory write operation
            Logger::get_instance().log(LogLevel::Info, &format!("Writing value {:?} to address {}", value, address), None);

            // Convert value to bytes and write to memory
            let value_bytes = value.to_bytes();

            // Perform the memory write operation
            MemoryWriter::get_instance().write_bytes(process_info.handle, *address, &value_bytes);
        } else {
            Logger::get_instance().log(LogLevel::Info, "No process is opened to write to.", None);
        }
    }
}
