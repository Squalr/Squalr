pub mod process;
pub mod scan;
pub mod project;

use crate::command::Command;

pub fn handle_commands(command: Command) {
    match command {
        Command::Process(cmd) => process::handle_process_command(cmd),
        Command::Scan(cmd) => scan::handle_scan_command(cmd),
        Command::Project(cmd) => project::handle_project_command(cmd),
    }
}
