use crate::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use crate::commands::{project_items::project_items_response::ProjectItemsResponse, unprivileged_command_response::UnprivilegedCommandResponse};
use crate::structures::projects::project_info::ProjectInfo;
use crate::structures::projects::project_items::project_item::ProjectItem;
use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectItemsListResponse {
    pub opened_project_info: Option<ProjectInfo>,
    pub opened_project_root: Option<ProjectItem>,
    pub opened_project_items: Vec<(ProjectItemRef, ProjectItem)>,
}

impl TypedUnprivilegedCommandResponse for ProjectItemsListResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::List {
            project_items_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::List { project_items_list_response }) = response {
            Ok(project_items_list_response)
        } else {
            Err(response)
        }
    }
}
