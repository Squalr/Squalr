use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::memory::read::memory_read_response::MemoryReadResponse;
use serde::Deserialize;
use serde::Serialize;
use squalr_engine_common::conversions::Conversions;
// use squalr_engine_common::structures::dynamic_struct::dynamic_struct::DynamicStruct;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct MemoryReadRequest {
    #[structopt(short = "a", long, parse(try_from_str = Conversions::parse_hex_or_int))]
    pub address: u64,
    // #[structopt(short = "v", long)]
    // pub value: DynamicStruct,
}

impl EngineRequest for MemoryReadRequest {
    type ResponseType = MemoryReadResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Memory(MemoryCommand::Read {
            memory_read_request: self.clone(),
        })
    }
}

impl From<MemoryReadResponse> for MemoryResponse {
    fn from(memory_read_response: MemoryReadResponse) -> Self {
        MemoryResponse::Read { memory_read_response }
    }
}
