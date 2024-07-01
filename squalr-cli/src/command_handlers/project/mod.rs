pub mod project_command;
pub use project_command::ProjectCommand;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::logging::log_level::LogLevel;

pub fn handle_project_command(cmd: ProjectCommand) {
    match cmd {
        ProjectCommand::List => {
            Logger::instance().log(LogLevel::Info, "Listing all projects", None);
        }
    }
}
