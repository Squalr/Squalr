use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectItemsDuplicateResponse {
    pub success: bool,
    pub duplicated_project_item_count: u64,
    pub duplicated_project_item_paths: Vec<PathBuf>,
}

impl TypedUnprivilegedCommandResponse for ProjectItemsDuplicateResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::Duplicate {
            project_items_duplicate_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::Duplicate {
            project_items_duplicate_response,
        }) = response
        {
            Ok(project_items_duplicate_response)
        } else {
            Err(response)
        }
    }
}
