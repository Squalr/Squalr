use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::commands::{project::create::project_create_response::ProjectCreateResponse, unprivileged_command::UnprivilegedCommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectCreateRequest {
    #[structopt(short = "p", long)]
    pub project_path: Option<PathBuf>,

    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
}

impl UnprivilegedCommandRequest for ProjectCreateRequest {
    type ResponseType = ProjectCreateResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::Project(ProjectCommand::Create {
            project_create_request: self.clone(),
        })
    }
}

impl From<ProjectCreateResponse> for ProjectResponse {
    fn from(project_create_response: ProjectCreateResponse) -> Self {
        ProjectResponse::Create { project_create_response }
    }
}
