use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsRenameModuleResponse {
    pub success: bool,
    pub module_name: String,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsRenameModuleResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::RenameModule {
            project_symbols_rename_module_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::RenameModule {
            project_symbols_rename_module_response,
        }) = response
        {
            Ok(project_symbols_rename_module_response)
        } else {
            Err(response)
        }
    }
}
