use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsSetCatalogResponse {
    pub success: bool,
    pub error: Option<String>,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsSetCatalogResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::SetCatalog {
            project_symbols_set_catalog_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::SetCatalog {
            project_symbols_set_catalog_response,
        }) = response
        {
            Ok(project_symbols_set_catalog_response)
        } else {
            Err(response)
        }
    }
}
