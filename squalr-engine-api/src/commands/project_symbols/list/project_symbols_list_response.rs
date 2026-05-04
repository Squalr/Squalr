use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use crate::structures::projects::project_info::ProjectInfo;
use crate::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsListResponse {
    pub opened_project_info: Option<ProjectInfo>,
    pub project_symbol_catalog: Option<ProjectSymbolCatalog>,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsListResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::List {
            project_symbols_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::List { project_symbols_list_response }) = response {
            Ok(project_symbols_list_response)
        } else {
            Err(response)
        }
    }
}
