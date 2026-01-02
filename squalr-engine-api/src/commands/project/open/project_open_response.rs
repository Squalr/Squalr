use crate::structures::projects::project_info::ProjectInfo;
use crate::{
    commands::{
        project::project_response::ProjectResponse,
        unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse},
    },
    structures::projects::project_items::project_item::ProjectItem,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectOpenResponse {
    pub opened_project_info: Option<ProjectInfo>,
    pub opened_project_root: Option<ProjectItem>,
}

impl TypedUnprivilegedCommandResponse for ProjectOpenResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::Project(ProjectResponse::Open {
            project_open_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::Project(ProjectResponse::Open { project_open_response }) = response {
            Ok(project_open_response)
        } else {
            Err(response)
        }
    }
}
