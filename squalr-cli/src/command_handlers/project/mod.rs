pub mod project_command;
pub use project_command::ProjectCommand;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_project_command(cmd: &mut ProjectCommand) {
    match cmd {
        ProjectCommand::List => {
            Logger::get_instance().log(LogLevel::Info, "Listing all projects", None);
        }
    }
}
