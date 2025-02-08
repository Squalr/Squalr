use crate::commands::memory::memory_command::MemoryCommand;
use crate::responses::engine_response::EngineResponse;
use crate::responses::memory::memory_response::MemoryResponse;
use crate::squalr_engine::SqualrEngine;
use crate::squalr_session::SqualrSession;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use uuid::Uuid;

pub fn handle_memory_read(
    cmd: MemoryCommand,
    uuid: Uuid,
) {
    if let MemoryCommand::Read { address, value } = cmd {
        if let Some(process_info) = SqualrSession::get_opened_process() {
            Logger::get_instance().log(LogLevel::Info, &format!("Reading value from address {}", address), None);

            let mut out_value = value.clone();
            let success = MemoryReader::get_instance().read(&process_info, address, &mut out_value);
            let response = EngineResponse::Memory(MemoryResponse::Read {
                value: out_value,
                address: address,
                success: success,
            });

            SqualrEngine::dispatch_response(response, uuid);
        } else {
            Logger::get_instance().log(LogLevel::Error, "No opened process available.", None);
        }
    }
}
