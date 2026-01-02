use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::memory::write::memory_write_response::MemoryWriteResponse;
use crate::conversions::conversions::Conversions;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct MemoryWriteRequest {
    // JIRA: Seems sus to just have generic int or hex parser.
    #[structopt(short = "a", long, parse(try_from_str = Conversions::parse_hex_or_int))]
    pub address: u64,

    #[structopt(short = "m")]
    pub module_name: String,

    #[structopt(short = "v")]
    pub value: Vec<u8>,
}

impl PrivilegedCommandRequest for MemoryWriteRequest {
    type ResponseType = MemoryWriteResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Memory(MemoryCommand::Write {
            memory_write_request: self.clone(),
        })
    }
}

impl From<MemoryWriteResponse> for MemoryResponse {
    fn from(memory_write_response: MemoryWriteResponse) -> Self {
        MemoryResponse::Write { memory_write_response }
    }
}
