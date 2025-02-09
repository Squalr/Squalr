use crate::commands::command_handler::CommandHandler;
use crate::commands::memory::handlers::memory_command_read::handle_memory_read;
use crate::commands::memory::handlers::memory_command_write::handle_memory_write;
use serde::Deserialize;
use serde::Serialize;
use squalr_engine_common::conversions::parse_hex_or_int;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum MemoryCommand {
    Read {
        #[structopt(short = "a", long, parse(try_from_str = parse_hex_or_int))]
        address: u64,
        #[structopt(short = "v", long)]
        value: DynamicStruct,
    },
    Write {
        #[structopt(short = "a", long, parse(try_from_str = parse_hex_or_int))]
        address: u64,
        #[structopt(short = "v", long)]
        value: DynamicStruct,
    },
}

impl CommandHandler for MemoryCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            MemoryCommand::Write { address, value } => {
                handle_memory_read(*address, value, uuid);
            }
            MemoryCommand::Read { address, value } => {
                handle_memory_write(*address, value, uuid);
            }
        }
    }
}
