pub mod process_command;
pub mod process_command_list;
pub mod process_command_open;
pub mod process_command_close;

pub use process_command::ProcessCommand;
pub use process_command_list::handle_process_list;
pub use process_command_open::handle_process_open;
pub use process_command_close::handle_process_close;

type ProcessCommandHandler = fn(ProcessCommand);

pub fn handle_process_command(cmd: ProcessCommand) {
    let handlers: &[(ProcessCommand, ProcessCommandHandler)] = &[
        (ProcessCommand::Open { pid: 0 }, handle_process_open),
        (ProcessCommand::List { windowed: false, search_term: None, match_case: false, system_processes: false, limit: None }, handle_process_list),
        (ProcessCommand::Close { pid: 0 }, handle_process_close),
    ];

    for (command, handler) in handlers {
        if std::mem::discriminant(&cmd) == std::mem::discriminant(command) {
            handler(cmd);
            return;
        }
    }
}
