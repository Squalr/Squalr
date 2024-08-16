use crate::command::Command;

pub mod memory;
pub mod process;
pub mod project;
pub mod scan;


pub fn handle_commands(command: &mut Command) {
    match command {
        Command::Memory(cmd) => memory::handle_memory_command(cmd),
        Command::Process(cmd) => process::handle_process_command(cmd),
        Command::Scan(cmd) => scan::handle_scan_command(cmd),
        Command::Project(cmd) => project::handle_project_command(cmd),
    }
}
