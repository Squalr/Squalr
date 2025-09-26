use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::structures::projects::project_info::ProjectInfo;
use crate::structures::projects::project_items::project_item::ProjectItem;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectItemsListResponse {
    pub opened_project_info: Option<ProjectInfo>,
    pub opened_project_root: Option<ProjectItem>,
}

impl TypedEngineCommandResponse for ProjectItemsListResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::ProjectItems(ProjectItemsResponse::List {
            project_items_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::ProjectItems(ProjectItemsResponse::List { project_items_list_response }) = response {
            Ok(project_items_list_response)
        } else {
            Err(response)
        }
    }
}
