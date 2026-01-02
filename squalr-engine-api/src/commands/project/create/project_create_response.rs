use crate::{
    commands::{
        project::project_response::ProjectResponse,
        unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse},
    },
    structures::projects::project_info::ProjectInfo,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectCreateResponse {
    pub created_project_info: Option<ProjectInfo>,
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
