pub mod process_command_close;
pub mod process_command_list;
pub mod process_command_open;

use crate::command_handlers::process::process_command_close::handle_process_close;
use crate::command_handlers::process::process_command_list::handle_process_list;
use crate::command_handlers::process::process_command_open::handle_process_open;
use crate::commands::process::process_command::ProcessCommand;
use uuid::Uuid;

pub fn handle_process_command(
    cmd: ProcessCommand,
    uuid: Uuid,
) {
    match cmd {
        ProcessCommand::Open { .. } => {
            handle_process_open(cmd, uuid);
        }
        ProcessCommand::List { .. } => {
            handle_process_list(cmd, uuid);
        }
        ProcessCommand::Close => {
            handle_process_close(cmd, uuid);
        }
    }
}
