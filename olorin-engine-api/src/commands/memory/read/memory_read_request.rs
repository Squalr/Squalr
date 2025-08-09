use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::memory::read::memory_read_response::MemoryReadResponse;
use crate::conversions::conversions::Conversions;
use crate::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use serde::Deserialize;
use serde::Serialize;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct MemoryReadRequest {
    // JIRA: Should probably bias hex and fall back on int? Maybe this can be made more explicit.
    #[structopt(short = "a", long, parse(try_from_str = Conversions::parse_hex_or_int))]
    pub address: u64,

    #[structopt(short = "m")]
    pub module_name: String,

    #[structopt(short = "v")]
    pub symbolic_struct_definition: SymbolicStructDefinition,
}

impl EngineCommandRequest for MemoryReadRequest {
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
