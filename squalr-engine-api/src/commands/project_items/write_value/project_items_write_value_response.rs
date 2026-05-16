use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectItemsWriteValueResponse {
    pub success: bool,
    pub error: Option<String>,
}

impl TypedUnprivilegedCommandResponse for ProjectItemsWriteValueResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::WriteValue {
            project_items_write_value_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::WriteValue {
            project_items_write_value_response,
        }) = response
        {
            Ok(project_items_write_value_response)
        } else {
            Err(response)
        }
    }
}
