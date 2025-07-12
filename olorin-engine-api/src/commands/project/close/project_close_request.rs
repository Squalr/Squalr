use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::project::close::project_close_response::ProjectCloseResponse;
use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project::project_response::ProjectResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectCloseRequest {}

impl EngineCommandRequest for ProjectCloseRequest {
    type ResponseType = ProjectCloseResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Project(ProjectCommand::Close {
            project_close_request: self.clone(),
        })
    }
}

impl From<ProjectCloseResponse> for ProjectResponse {
    fn from(project_close_response: ProjectCloseResponse) -> Self {
        ProjectResponse::Close { project_close_response }
    }
}
