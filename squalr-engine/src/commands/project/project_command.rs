use crate::commands::command_handler::CommandHandler;
use crate::commands::project::handlers::project_command_list::handle_project_list;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectCommand {
    /// List all projects
    List,
    // Add other project commands here
}

impl CommandHandler for ProjectCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            ProjectCommand::List {} => {
                handle_project_list(uuid);
            }
        }
    }
}
