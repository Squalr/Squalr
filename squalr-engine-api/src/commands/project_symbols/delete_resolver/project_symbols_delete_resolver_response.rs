use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsDeleteResolverResponse {
    pub success: bool,
    pub resolver_id: String,
    pub error: Option<String>,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsDeleteResolverResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::DeleteResolver {
            project_symbols_delete_resolver_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::DeleteResolver {
            project_symbols_delete_resolver_response,
        }) = response
        {
            Ok(project_symbols_delete_resolver_response)
        } else {
            Err(response)
        }
    }
}
