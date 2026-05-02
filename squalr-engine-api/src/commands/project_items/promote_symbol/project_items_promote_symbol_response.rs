use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectItemsPromoteSymbolConflict {
    pub project_item_path: PathBuf,
    pub symbol_locator_key: String,
    pub existing_display_name: String,
    pub existing_locator_display: String,
    pub requested_display_name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectItemsPromoteSymbolResponse {
    pub success: bool,
    pub promoted_symbol_count: u64,
    pub reused_symbol_count: u64,
    pub promoted_symbol_locator_keys: Vec<String>,
    pub conflicts: Vec<ProjectItemsPromoteSymbolConflict>,
}

impl TypedUnprivilegedCommandResponse for ProjectItemsPromoteSymbolResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::PromoteSymbol {
            project_items_promote_symbol_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectItems(ProjectItemsResponse::PromoteSymbol {
            project_items_promote_symbol_response,
        }) = response
        {
            Ok(project_items_promote_symbol_response)
        } else {
            Err(response)
        }
    }
}
