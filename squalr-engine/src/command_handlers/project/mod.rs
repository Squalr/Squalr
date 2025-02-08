use crate::commands::project::project_command::ProjectCommand;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use uuid::Uuid;

pub fn handle_project_command(
    cmd: ProjectCommand,
    uuid: Uuid,
) {
    match cmd {
        ProjectCommand::List => {
            Logger::get_instance().log(LogLevel::Info, "Listing all projects", None);
        }
    }
}
