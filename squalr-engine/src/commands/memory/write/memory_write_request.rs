use std::sync::Arc;

use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::memory::write::memory_write_response::MemoryWriteResponse;
use crate::engine_execution_context::EngineExecutionContext;
use serde::{Deserialize, Serialize};
use squalr_engine_common::conversions::parse_hex_or_int;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use squalr_engine_common::dynamic_struct::to_bytes::ToBytes;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_memory::memory_writer::MemoryWriter;
use squalr_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct MemoryWriteRequest {
    #[structopt(short = "a", long, parse(try_from_str = parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "v", long)]
    pub value: DynamicStruct,
}

impl EngineRequest for MemoryWriteRequest {
    type ResponseType = MemoryWriteResponse;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> Self::ResponseType {
        if let Some(process_info) = execution_context.get_opened_process() {
            // Log the memory write operation
            Logger::log(LogLevel::Info, &format!("Writing value {:?} to address {}", self.value, self.address), None);

            // Convert value to bytes and write to memory
            let value_bytes = self.value.to_bytes();

            // Perform the memory write operation
            let success = MemoryWriter::get_instance().write_bytes(process_info.handle, self.address, &value_bytes);

            MemoryWriteResponse { success }
        } else {
            Logger::log(LogLevel::Info, "No process is opened to write to.", None);
            MemoryWriteResponse { success: false }
        }
    }

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Memory(MemoryCommand::Write {
            memory_write_request: self.clone(),
        })
    }
}

impl From<MemoryWriteResponse> for MemoryResponse {
    fn from(memory_write_response: MemoryWriteResponse) -> Self {
        MemoryResponse::Write { memory_write_response }
    }
}
