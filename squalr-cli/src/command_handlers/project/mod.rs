pub mod project_command;

pub use project_command::ProjectCommand;

pub fn handle_project_command(cmd: ProjectCommand) {
    match cmd {
        ProjectCommand::List => {
            println!("Listing all projects");
            // Implement the logic for listing projects
        }
        // Handle other project commands here
    }
}
