use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::project::project_response::ProjectResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectCloseResponse {}

impl TypedEngineCommandResponse for ProjectCloseResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Project(ProjectResponse::Close {
            project_close_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Project(ProjectResponse::Close { project_close_response }) = response {
            Ok(project_close_response)
        } else {
            Err(response)
        }
    }
}
