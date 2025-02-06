pub mod memory_command_read;
pub mod memory_command_write;

use crate::command_handlers::memory::memory_command_read::handle_memory_read;
use crate::command_handlers::memory::memory_command_write::handle_memory_write;
use crate::commands::memory::memory_command::MemoryCommand;

pub fn handle_memory_command(cmd: &mut MemoryCommand) {
    match cmd {
        MemoryCommand::Read { .. } => handle_memory_read(cmd),
        MemoryCommand::Write { .. } => handle_memory_write(cmd),
    }
}
