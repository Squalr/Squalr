use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectItemsPromoteSymbolResponse {
    pub success: bool,
    pub promoted_symbol_count: u64,
    pub promoted_symbol_keys: Vec<String>,
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
