use crate::commands::engine_request::EngineRequest;
use crate::commands::engine_response::{EngineResponse, TypedEngineResponse};
use crate::commands::memory::read::memory_read_request::MemoryReadRequest;
use crate::commands::memory::write::memory_write_request::MemoryWriteRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum MemoryCommand {
    Read {
        #[structopt(flatten)]
        memory_read_request: MemoryReadRequest,
    },
    Write {
        #[structopt(flatten)]
        memory_write_request: MemoryWriteRequest,
    },
}

impl MemoryCommand {
    pub fn execute(&self) -> EngineResponse {
        match self {
            MemoryCommand::Write { memory_write_request } => memory_write_request.execute().to_engine_response(),
            MemoryCommand::Read { memory_read_request } => memory_read_request.execute().to_engine_response(),
        }
    }
}
