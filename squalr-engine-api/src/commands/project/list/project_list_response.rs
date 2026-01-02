use crate::commands::{
    project::project_response::ProjectResponse,
    unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse},
};
use crate::structures::projects::project_info::ProjectInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectListResponse {
    pub projects_info: Vec<ProjectInfo>,
}

impl TypedUnprivilegedCommandResponse for ProjectListResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::Project(ProjectResponse::List {
            project_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::Project(ProjectResponse::List { project_list_response }) = response {
            Ok(project_list_response)
        } else {
            Err(response)
        }
    }
}
