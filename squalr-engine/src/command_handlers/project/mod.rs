use crate::commands::project::project_command::ProjectCommand;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

pub fn handle_project_command(cmd: ProjectCommand) {
    match cmd {
        ProjectCommand::List => {
            Logger::get_instance().log(LogLevel::Info, "Listing all projects", None);
        }
    }
}
