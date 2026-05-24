use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsCreateModuleResponse {
    pub success: bool,
    pub module_name: String,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsCreateModuleResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::CreateModule {
            project_symbols_create_module_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::CreateModule {
            project_symbols_create_module_response,
        }) = response
        {
            Ok(project_symbols_create_module_response)
        } else {
            Err(response)
        }
    }
}
