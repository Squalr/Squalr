use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::commands::{project::list::project_list_response::ProjectListResponse, unprivileged_command::UnprivilegedCommand};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectListRequest {}

impl UnprivilegedCommandRequest for ProjectListRequest {
    type ResponseType = ProjectListResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::Project(ProjectCommand::List {
            project_list_request: self.clone(),
        })
    }
}

impl From<ProjectListResponse> for ProjectResponse {
    fn from(project_list_response: ProjectListResponse) -> Self {
        ProjectResponse::List { project_list_response }
    }
}
