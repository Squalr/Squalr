use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::commands::{project::delete::project_delete_response::ProjectDeleteResponse, unprivileged_command::UnprivilegedCommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectDeleteRequest {
    #[structopt(short = "p", long)]
    pub project_directory_path: Option<PathBuf>,

    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
}

impl UnprivilegedCommandRequest for ProjectDeleteRequest {
    type ResponseType = ProjectDeleteResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::Project(ProjectCommand::Delete {
            project_delete_request: self.clone(),
        })
    }
}

impl From<ProjectDeleteResponse> for ProjectResponse {
    fn from(project_delete_response: ProjectDeleteResponse) -> Self {
        ProjectResponse::Delete { project_delete_response }
    }
}
