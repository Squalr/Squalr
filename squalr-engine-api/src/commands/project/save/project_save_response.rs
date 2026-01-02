use crate::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use crate::commands::{project::project_response::ProjectResponse, unprivileged_command_response::UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSaveResponse {
    pub success: bool,
}

impl TypedUnprivilegedCommandResponse for ProjectSaveResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::Project(ProjectResponse::Save {
            project_save_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::Project(ProjectResponse::Save { project_save_response }) = response {
            Ok(project_save_response)
        } else {
            Err(response)
        }
    }
}
