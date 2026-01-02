use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::commands::{project::close::project_close_response::ProjectCloseResponse, unprivileged_command::UnprivilegedCommand};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectCloseRequest {}

impl UnprivilegedCommandRequest for ProjectCloseRequest {
    type ResponseType = ProjectCloseResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::Project(ProjectCommand::Close {
            project_close_request: self.clone(),
        })
    }
}

impl From<ProjectCloseResponse> for ProjectResponse {
    fn from(project_close_response: ProjectCloseResponse) -> Self {
        ProjectResponse::Close { project_close_response }
    }
}
