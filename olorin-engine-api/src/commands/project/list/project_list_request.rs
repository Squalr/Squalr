use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::{engine_command::EngineCommand, project::list::project_list_response::ProjectListResponse};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectListRequest {}

impl EngineCommandRequest for ProjectListRequest {
    type ResponseType = ProjectListResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Project(ProjectCommand::List {
            project_list_request: self.clone(),
        })
    }
}

impl From<ProjectListResponse> for ProjectResponse {
    fn from(project_list_response: ProjectListResponse) -> Self {
        ProjectResponse::List { project_list_response }
    }
}
