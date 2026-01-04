use crate::commands::{
    project::project_response::ProjectResponse,
    unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectDeleteResponse {
    pub success: bool,
}

impl TypedUnprivilegedCommandResponse for ProjectDeleteResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::Project(ProjectResponse::Delete {
            project_delete_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::Project(ProjectResponse::Delete { project_delete_response }) = response {
            Ok(project_delete_response)
        } else {
            Err(response)
        }
    }
}
