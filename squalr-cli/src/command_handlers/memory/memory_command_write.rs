use crate::command_handlers::memory::memory_command::MemoryCommand;
use crate::session_manager::SESSION_MANAGER;
use squalr_engine_common::logging::logger::LOGGER;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_memory::memory_writer::MemoryWriter;
use squalr_engine_common::dynamic_struct::to_bytes::ToBytes;

pub fn handle_memory_write(cmd: MemoryCommand) {
    if let MemoryCommand::Write { address, value } = cmd {
        let session_manager = SESSION_MANAGER.lock().unwrap();
        if let Some(opened_pid) = session_manager.get_opened_process() {
            // Log the memory write operation
            LOGGER.log(
                LogLevel::Info,
                &format!(
                    "Writing value {:?} to address {}",
                    value, address
                ),
                None
            );

            // Perform the memory write operation
            let memory_writer = MemoryWriter::instance();
            let memory_writer = memory_writer.lock().expect("Failed to acquire memory writer lock");
    
            // Convert value to bytes and write to memory
            let value_bytes = value.to_bytes();
            memory_writer.write_bytes(&opened_pid, address, &value_bytes);
        } else {
            LOGGER.log(LogLevel::Info, "No process is opened to write to.", None);
        }
    }
}
