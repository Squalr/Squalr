use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectItemsRenameResponse {
    pub success: bool,
    pub renamed_project_item_path: PathBuf,
}

impl TypedUnprivilegedCommandResponse for ProjectItemsRenameResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::Rename {
            project_items_rename_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::Rename { project_items_rename_response }) = response {
            Ok(project_items_rename_response)
        } else {
            Err(response)
        }
    }
}
