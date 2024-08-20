pub mod memory_command;
pub mod memory_command_read;
pub mod memory_command_write;

pub use memory_command::MemoryCommand;
pub use memory_command_read::handle_memory_read;
pub use memory_command_write::handle_memory_write;

pub fn handle_memory_command(
    cmd: &mut MemoryCommand,
) {
    match cmd {
        MemoryCommand::Read { .. } => handle_memory_read(cmd),
        MemoryCommand::Write { .. } => handle_memory_write(cmd),
    }
}
