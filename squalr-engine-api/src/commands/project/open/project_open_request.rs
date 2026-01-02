use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::commands::{project::open::project_open_response::ProjectOpenResponse, unprivileged_command::UnprivilegedCommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectOpenRequest {
    #[structopt(short = "p", long)]
    pub project_path: Option<PathBuf>,

    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
}

impl UnprivilegedCommandRequest for ProjectOpenRequest {
    type ResponseType = ProjectOpenResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::Project(ProjectCommand::Open {
            project_open_request: self.clone(),
        })
    }
}

impl From<ProjectOpenResponse> for ProjectResponse {
    fn from(project_open_response: ProjectOpenResponse) -> Self {
        ProjectResponse::Open { project_open_response }
    }
}
