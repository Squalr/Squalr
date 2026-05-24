use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsUpdateResponse {
    pub success: bool,
    pub symbol_locator_key: String,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsUpdateResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::Update {
            project_symbols_update_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::Update {
            project_symbols_update_response,
        }) = response
        {
            Ok(project_symbols_update_response)
        } else {
            Err(response)
        }
    }
}
