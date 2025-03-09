use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::memory::write::memory_write_response::MemoryWriteResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_common::conversions::Conversions;
// use squalr_engine_common::{conversions::Conversions, structures::dynamic_struct::dynamic_struct::DynamicStruct};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct MemoryWriteRequest {
    #[structopt(short = "a", long, parse(try_from_str = Conversions::parse_hex_or_int))]
    pub address: u64,
    // #[structopt(short = "v", long)]
    // pub value: DynamicStruct,
}

impl EngineCommandRequest for MemoryWriteRequest {
    type ResponseType = MemoryWriteResponse;

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
