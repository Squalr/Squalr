use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsCreateResponse {
    pub success: bool,
    pub created_symbol_locator_key: String,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsCreateResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::Create {
            project_symbols_create_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::Create {
            project_symbols_create_response,
        }) = response
        {
            Ok(project_symbols_create_response)
        } else {
            Err(response)
        }
    }
}
