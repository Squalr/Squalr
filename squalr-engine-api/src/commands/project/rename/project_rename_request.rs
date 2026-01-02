use crate::commands::project::project_response::ProjectResponse;
use crate::commands::project::rename::project_rename_response::ProjectRenameResponse;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::commands::{project::project_command::ProjectCommand, unprivileged_command::UnprivilegedCommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectRenameRequest {
    #[structopt(short = "p", long)]
    pub project_path: PathBuf,

    #[structopt(short = "n", long)]
    pub new_project_name: String,
}

impl UnprivilegedCommandRequest for ProjectRenameRequest {
    type ResponseType = ProjectRenameResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::Project(ProjectCommand::Rename {
            project_rename_request: self.clone(),
        })
    }
}

impl From<ProjectRenameResponse> for ProjectResponse {
    fn from(project_rename_response: ProjectRenameResponse) -> Self {
        ProjectResponse::Rename { project_rename_response }
    }
}
