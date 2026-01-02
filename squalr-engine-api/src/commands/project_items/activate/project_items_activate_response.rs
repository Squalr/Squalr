use crate::commands::unprivileged_command_response::UnprivilegedCommandResponse;
use crate::commands::{project_items::project_items_response::ProjectItemsResponse, unprivileged_command_response::TypedUnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectItemsActivateResponse {}

impl TypedUnprivilegedCommandResponse for ProjectItemsActivateResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::Activate {
            project_items_activate_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::Activate {
            project_items_activate_response,
        }) = response
        {
            Ok(project_items_activate_response)
        } else {
            Err(response)
        }
    }
}
