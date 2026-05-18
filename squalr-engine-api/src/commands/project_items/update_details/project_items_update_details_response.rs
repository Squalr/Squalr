use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectItemsUpdateDetailsResponse {
    pub success: bool,
    pub updated_project_item_count: u64,
    pub error: Option<String>,
}

impl TypedUnprivilegedCommandResponse for ProjectItemsUpdateDetailsResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::UpdateDetails {
            project_items_update_details_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::UpdateDetails {
            project_items_update_details_response,
        }) = response
        {
            Ok(project_items_update_details_response)
        } else {
            Err(response)
        }
    }
}
