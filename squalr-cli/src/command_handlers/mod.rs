use crate::command::Command;

pub mod memory;
pub mod process;
pub mod project;
pub mod scan;


pub async fn handle_commands(command: &mut Command) {
    match command {
        Command::Memory(cmd) => memory::handle_memory_command(cmd).await,
        Command::Process(cmd) => process::handle_process_command(cmd).await,
        Command::Scan(cmd) => scan::handle_scan_command(cmd).await,
        Command::Project(cmd) => project::handle_project_command(cmd).await,
    }
}
