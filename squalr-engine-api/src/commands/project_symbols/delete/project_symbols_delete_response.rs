use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsDeleteResponse {
    pub success: bool,
    pub deleted_symbol_count: u64,
    pub deleted_module_count: u64,
    pub deleted_module_range_count: u64,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsDeleteResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::Delete {
            project_symbols_delete_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::Delete {
            project_symbols_delete_response,
        }) = response
        {
            Ok(project_symbols_delete_response)
        } else {
            Err(response)
        }
    }
}
