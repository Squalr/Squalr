use crate::command_handlers::process::process_command::ProcessCommand;

pub fn handle_process_close(cmd: ProcessCommand) {
    if let ProcessCommand::Close { pid } = cmd {
        println!("Closing process with PID: {}", pid);
        // Implement the logic for closing a process
    }
}
