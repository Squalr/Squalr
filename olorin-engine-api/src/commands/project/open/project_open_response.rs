use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::project::project_response::ProjectResponse;
use crate::structures::projects::project_info::ProjectInfo;
use crate::structures::projects::project_items::project_item::ProjectItem;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectOpenResponse {
    pub opened_project_info: Option<ProjectInfo>,
    pub opened_project_root: Option<ProjectItem>,
}

impl TypedEngineCommandResponse for ProjectOpenResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Project(ProjectResponse::Open {
            project_open_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Project(ProjectResponse::Open { project_open_response }) = response {
            Ok(project_open_response)
        } else {
            Err(response)
        }
    }
}
