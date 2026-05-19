use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsUpsertLayoutResponse {
    pub success: bool,
    pub struct_layout_id: String,
    pub error: Option<String>,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsUpsertLayoutResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::UpsertLayout {
            project_symbols_upsert_layout_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::UpsertLayout {
            project_symbols_upsert_layout_response,
        }) = response
        {
            Ok(project_symbols_upsert_layout_response)
        } else {
            Err(response)
        }
    }
}
