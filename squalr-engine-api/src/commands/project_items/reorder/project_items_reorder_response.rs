use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectItemsReorderResponse {
    pub success: bool,
    pub reordered_project_item_count: u64,
}

impl TypedUnprivilegedCommandResponse for ProjectItemsReorderResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::Reorder {
            project_items_reorder_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::Reorder {
            project_items_reorder_response,
        }) = response
        {
            Ok(project_items_reorder_response)
        } else {
            Err(response)
        }
    }
}
