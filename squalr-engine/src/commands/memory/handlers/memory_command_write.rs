use crate::responses::engine_response::EngineResponse;
use crate::responses::memory::memory_response::MemoryResponse;
use crate::squalr_engine::SqualrEngine;
use crate::squalr_session::SqualrSession;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use squalr_engine_common::dynamic_struct::to_bytes::ToBytes;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_memory::memory_writer::MemoryWriter;
use squalr_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
use uuid::Uuid;

pub fn handle_memory_write(
    address: u64,
    value: &DynamicStruct,
    uuid: Uuid,
) {
    if let Some(process_info) = SqualrSession::get_opened_process() {
        // Log the memory write operation
        Logger::get_instance().log(LogLevel::Info, &format!("Writing value {:?} to address {}", value, address), None);

        // Convert value to bytes and write to memory
        let value_bytes = value.to_bytes();

        // Perform the memory write operation
        let success = MemoryWriter::get_instance().write_bytes(process_info.handle, address, &value_bytes);
        let response = EngineResponse::Memory(MemoryResponse::Write { success: success });

        SqualrEngine::dispatch_response(response, uuid);
    } else {
        Logger::get_instance().log(LogLevel::Info, "No process is opened to write to.", None);
    }
}
