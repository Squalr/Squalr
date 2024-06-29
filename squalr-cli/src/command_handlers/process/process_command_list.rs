use crate::command_handlers::process::process_command::ProcessCommand;

pub fn handle_process_list(cmd: ProcessCommand) {
    if let ProcessCommand::List { windowed, search_term, match_case, system_processes, limit } = cmd {
        println!(
            "Listing processes with options: windowed={}, search_term={:?}, match_case={}, system_processes={}, limit={:?}",
            windowed, search_term, match_case, system_processes, limit
        );
        // Implement the logic for listing processes
    }
}