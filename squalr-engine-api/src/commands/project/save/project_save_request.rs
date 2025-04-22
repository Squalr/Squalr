use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::project::save::project_save_response::ProjectSaveResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSaveRequest {}

impl EngineCommandRequest for ProjectSaveRequest {
    type ResponseType = ProjectSaveResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Project(ProjectCommand::Save {
            project_save_request: self.clone(),
        })
    }
}

impl From<ProjectSaveResponse> for ProjectResponse {
    fn from(project_save_response: ProjectSaveResponse) -> Self {
        ProjectResponse::Save { project_save_response }
    }
}
