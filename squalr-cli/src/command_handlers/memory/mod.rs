use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;

pub mod memory_command;
pub mod memory_command_read;
pub mod memory_command_write;

pub use memory_command::MemoryCommand;
pub use memory_command_read::handle_memory_read;
pub use memory_command_write::handle_memory_write;

type MemoryCommandHandler = fn(MemoryCommand);

pub fn handle_memory_command(cmd: MemoryCommand) {
    let handlers: &[(MemoryCommand, MemoryCommandHandler)] = &[
        (MemoryCommand::Read { address: 0, value: DynamicStruct::new() }, handle_memory_read),
        (MemoryCommand::Write { address: 0, value: DynamicStruct::new() }, handle_memory_write),
    ];

    for (command, handler) in handlers {
        if std::mem::discriminant(&cmd) == std::mem::discriminant(command) {
            handler(cmd);
            return;
        }
    }
}
