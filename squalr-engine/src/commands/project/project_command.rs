use crate::commands::command_handler::CommandHandler;
use crate::commands::project::requests::project_list_request::ProjectListRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectCommand {
    /// List all projects
    List {
        #[structopt(flatten)]
        project_list_request: ProjectListRequest,
    },
}

impl CommandHandler for ProjectCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            ProjectCommand::List { project_list_request } => {
                project_list_request.handle(uuid);
            }
        }
    }
}
