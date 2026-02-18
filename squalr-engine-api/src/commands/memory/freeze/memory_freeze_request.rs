use crate::commands::memory::freeze::memory_freeze_response::MemoryFreezeResponse;
use crate::commands::memory::freeze::memory_freeze_target::MemoryFreezeTarget;
use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Default, Serialize, Deserialize)]
pub struct MemoryFreezeRequest {
    #[structopt(skip)]
    pub freeze_targets: Vec<MemoryFreezeTarget>,
    #[structopt(short = "f", long = "frozen")]
    pub is_frozen: bool,
}

impl PrivilegedCommandRequest for MemoryFreezeRequest {
    type ResponseType = MemoryFreezeResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Memory(MemoryCommand::Freeze {
            memory_freeze_request: self.clone(),
        })
    }
}

impl From<MemoryFreezeResponse> for MemoryResponse {
    fn from(memory_freeze_response: MemoryFreezeResponse) -> Self {
        MemoryResponse::Freeze { memory_freeze_response }
    }
}
