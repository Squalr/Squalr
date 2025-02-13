use crate::commands::command_handler::CommandHandler;
use crate::responses::engine_response::EngineResponse;
use crate::responses::memory::memory_response::MemoryResponse;
use crate::squalr_engine::SqualrEngine;
use crate::squalr_session::SqualrSession;
use serde::Deserialize;
use serde::Serialize;
use squalr_engine_common::conversions::parse_hex_or_int;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct MemoryReadRequest {
    #[structopt(short = "a", long, parse(try_from_str = parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "v", long)]
    pub value: DynamicStruct,
}

impl CommandHandler for MemoryReadRequest {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        if let Some(process_info) = SqualrSession::get_opened_process() {
            Logger::get_instance().log(LogLevel::Info, &format!("Reading value from address {}", self.address), None);

            let mut out_value = self.value.clone();
            let success = MemoryReader::get_instance().read(&process_info, self.address, &mut out_value);
            let response = EngineResponse::Memory(MemoryResponse::Read {
                value: out_value,
                address: self.address,
                success: success,
            });

            SqualrEngine::dispatch_response(response, uuid);
        } else {
            Logger::get_instance().log(LogLevel::Error, "No opened process available.", None);
        }
    }
}
