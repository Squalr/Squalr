use crate::commands::{
    project::project_response::ProjectResponse,
    unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectCreateResponse {
    pub success: bool,
    pub new_project_path: PathBuf,
}

impl TypedUnprivilegedCommandResponse for ProjectCreateResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::Project(ProjectResponse::Create {
            project_create_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::Project(ProjectResponse::Create { project_create_response }) = response {
            Ok(project_create_response)
        } else {
            Err(response)
        }
    }
}
