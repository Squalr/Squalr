use std::path::PathBuf;

use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::project::rename::project_rename_response::ProjectRenameResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectRenameRequest {
    #[structopt(short = "p", long)]
    pub project_path: PathBuf,

    #[structopt(short = "n", long)]
    pub new_project_name: String,
}

impl EngineCommandRequest for ProjectRenameRequest {
    type ResponseType = ProjectRenameResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Project(ProjectCommand::Rename {
            project_rename_request: self.clone(),
        })
    }
}

impl From<ProjectRenameResponse> for ProjectResponse {
    fn from(project_rename_response: ProjectRenameResponse) -> Self {
        ProjectResponse::Rename { project_rename_response }
    }
}
