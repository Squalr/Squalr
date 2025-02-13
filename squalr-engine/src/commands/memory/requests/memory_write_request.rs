use crate::commands::command_handler::CommandHandler;
use crate::responses::engine_response::EngineResponse;
use crate::responses::memory::memory_response::MemoryResponse;
use crate::squalr_engine::SqualrEngine;
use crate::squalr_session::SqualrSession;
use serde::{Deserialize, Serialize};
use squalr_engine_common::conversions::parse_hex_or_int;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use squalr_engine_common::dynamic_struct::to_bytes::ToBytes;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_memory::memory_writer::MemoryWriter;
use squalr_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct MemoryWriteRequest {
    #[structopt(short = "a", long, parse(try_from_str = parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "v", long)]
    pub value: DynamicStruct,
}

impl CommandHandler for MemoryWriteRequest {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        if let Some(process_info) = SqualrSession::get_opened_process() {
            // Log the memory write operation
            Logger::get_instance().log(LogLevel::Info, &format!("Writing value {:?} to address {}", self.value, self.address), None);

            // Convert value to bytes and write to memory
            let value_bytes = self.value.to_bytes();

            // Perform the memory write operation
            let success = MemoryWriter::get_instance().write_bytes(process_info.handle, self.address, &value_bytes);
            let response = EngineResponse::Memory(MemoryResponse::Write { success: success });

            SqualrEngine::dispatch_response(response, uuid);
        } else {
            Logger::get_instance().log(LogLevel::Info, "No process is opened to write to.", None);
        }
    }
}
