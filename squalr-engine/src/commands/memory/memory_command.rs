use crate::commands::command_handler::CommandHandler;
use crate::commands::memory::requests::memory_read_request::MemoryReadRequest;
use crate::commands::memory::requests::memory_write_request::MemoryWriteRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

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

impl CommandHandler for MemoryCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            MemoryCommand::Write { memory_write_request } => {
                memory_write_request.handle(uuid);
            }
            MemoryCommand::Read { memory_read_request } => {
                memory_read_request.handle(uuid);
            }
        }
    }
}
