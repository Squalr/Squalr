use std::path::PathBuf;

use crate::commands::{
    project::project_response::ProjectResponse,
    unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectRenameResponse {
    pub success: bool,
    pub new_project_path: PathBuf,
}

impl TypedUnprivilegedCommandResponse for ProjectRenameResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::Project(ProjectResponse::Rename {
            project_rename_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::Project(ProjectResponse::Rename { project_rename_response }) = response {
            Ok(project_rename_response)
        } else {
            Err(response)
        }
    }
}
