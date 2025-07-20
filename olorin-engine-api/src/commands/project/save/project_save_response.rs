use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::project::project_response::ProjectResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSaveResponse {
    pub success: bool,
}

impl TypedEngineCommandResponse for ProjectSaveResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Project(ProjectResponse::Save {
            project_save_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Project(ProjectResponse::Save { project_save_response }) = response {
            Ok(project_save_response)
        } else {
            Err(response)
        }
    }
}
