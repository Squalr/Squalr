use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::project::project_response::ProjectResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectExportResponse {
    pub success: bool,
}

impl TypedEngineCommandResponse for ProjectExportResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Project(ProjectResponse::Export {
            project_export_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Project(ProjectResponse::Export { project_export_response }) = response {
            Ok(project_export_response)
        } else {
            Err(response)
        }
    }
}
