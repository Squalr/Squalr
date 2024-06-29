pub mod process_command;

pub use process_command::ProcessCommand;

pub fn handle_process_command(cmd: ProcessCommand) {
    match cmd {
        ProcessCommand::Open { pid } => {
            println!("Opening process with PID: {}", pid);
            // Implement the logic for opening a process
        }
        ProcessCommand::List { windowed, search_term, match_case, system_processes, limit } => {
            println!("Listing processes with options: windowed={}, search_term={:?}, match_case={}, system_processes={}, limit={:?}",
                windowed, search_term, match_case, system_processes, limit);
            // Implement the logic for listing processes
        }
        ProcessCommand::Close { pid } => {
            println!("Closing process with PID: {}", pid);
            // Implement the logic for closing a process
        }
        // Handle other process commands here
    }
}
