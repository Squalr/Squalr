use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsUpsertResolverResponse {
    pub success: bool,
    pub resolver_id: String,
    pub error: Option<String>,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsUpsertResolverResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::UpsertResolver {
            project_symbols_upsert_resolver_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::UpsertResolver {
            project_symbols_upsert_resolver_response,
        }) = response
        {
            Ok(project_symbols_upsert_resolver_response)
        } else {
            Err(response)
        }
    }
}
