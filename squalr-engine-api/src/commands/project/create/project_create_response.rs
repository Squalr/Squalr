use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::project::project_response::ProjectResponse;
use crate::structures::projects::project_info::ProjectInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectCreateResponse {
    pub created_project_info: Option<ProjectInfo>,
}

impl TypedEngineCommandResponse for ProjectCreateResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Project(ProjectResponse::Create {
            project_create_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Project(ProjectResponse::Create { project_create_response }) = response {
            Ok(project_create_response)
        } else {
            Err(response)
        }
    }
}
