use crate::command_handlers::process::process_command::ProcessCommand;

pub fn handle_process_open(cmd: ProcessCommand) {
    if let ProcessCommand::Open { pid } = cmd {
        println!("Opening process with PID: {}", pid);
        // Implement the logic for opening a process
    }
}
