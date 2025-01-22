pub mod process_command;
pub mod process_command_close;
pub mod process_command_list;
pub mod process_command_open;

pub use process_command::ProcessCommand;
pub use process_command_close::handle_process_close;
pub use process_command_list::handle_process_list;
pub use process_command_open::handle_process_open;

pub fn handle_process_command(cmd: &mut ProcessCommand) {
    match cmd {
        ProcessCommand::Open { .. } => {
            handle_process_open(cmd);
        }
        ProcessCommand::List { .. } => {
            handle_process_list(cmd);
        }
        ProcessCommand::Close => {
            handle_process_close(cmd);
        }
    }
}
