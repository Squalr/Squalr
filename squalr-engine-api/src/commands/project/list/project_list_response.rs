use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::project::project_response::ProjectResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectListResponse {}

impl TypedEngineCommandResponse for ProjectListResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Project(ProjectResponse::List {
            project_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Project(ProjectResponse::List { project_list_response }) = response {
            Ok(project_list_response)
        } else {
            Err(response)
        }
    }
}
