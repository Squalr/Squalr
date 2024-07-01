pub mod scan_command;
pub mod scan_command_value;
pub mod scan_command_collect;

pub use scan_command::ScanCommand;
pub use scan_command_value::handle_value_command;
pub use scan_command_collect::handle_collect_command;

type ScanCommandHandler = fn(ScanCommand);

pub fn handle_scan_command(cmd: ScanCommand) {
    let handlers: &[(ScanCommand, ScanCommandHandler)] = &[
        (ScanCommand::Value { value: 0 }, handle_value_command),
        (ScanCommand::Collect, handle_collect_command),
    ];

    for (command, handler) in handlers {
        if std::mem::discriminant(&cmd) == std::mem::discriminant(command) {
            handler(cmd);
            return;
        }
    }
}
