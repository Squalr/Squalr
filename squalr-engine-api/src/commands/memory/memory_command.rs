use crate::commands::memory::freeze::memory_freeze_request::MemoryFreezeRequest;
use crate::commands::memory::read::memory_read_request::MemoryReadRequest;
use crate::commands::memory::write::memory_write_request::MemoryWriteRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum MemoryCommand {
    Freeze {
        #[structopt(flatten)]
        memory_freeze_request: MemoryFreezeRequest,
    },
    Read {
        #[structopt(flatten)]
        memory_read_request: MemoryReadRequest,
    },
    Write {
        #[structopt(flatten)]
        memory_write_request: MemoryWriteRequest,
    },
}
