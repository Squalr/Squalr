use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsRenameResponse {
    pub success: bool,
    pub symbol_locator_key: String,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsRenameResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::Rename {
            project_symbols_rename_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::Rename {
            project_symbols_rename_response,
        }) = response
        {
            Ok(project_symbols_rename_response)
        } else {
            Err(response)
        }
    }
}
