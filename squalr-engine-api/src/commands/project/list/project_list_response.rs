use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::project::project_response::ProjectResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectListResponse {}

impl TypedEngineResponse for ProjectListResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Project(ProjectResponse::List {
            project_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Project(ProjectResponse::List { project_list_response }) = response {
            Ok(project_list_response)
        } else {
            Err(response)
        }
    }
}
