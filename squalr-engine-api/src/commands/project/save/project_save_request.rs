use crate::commands::project::project_response::ProjectResponse;
use crate::commands::project::save::project_save_response::ProjectSaveResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::{project::project_command::ProjectCommand, unprivileged_command_request::UnprivilegedCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSaveRequest {}

impl UnprivilegedCommandRequest for ProjectSaveRequest {
    type ResponseType = ProjectSaveResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::Project(ProjectCommand::Save {
            project_save_request: self.clone(),
        })
    }
}

impl From<ProjectSaveResponse> for ProjectResponse {
    fn from(project_save_response: ProjectSaveResponse) -> Self {
        ProjectResponse::Save { project_save_response }
    }
}
