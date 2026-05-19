use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsDeleteLayoutResponse {
    pub success: bool,
    pub struct_layout_id: String,
    pub error: Option<String>,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsDeleteLayoutResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::DeleteLayout {
            project_symbols_delete_layout_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::DeleteLayout {
            project_symbols_delete_layout_response,
        }) = response
        {
            Ok(project_symbols_delete_layout_response)
        } else {
            Err(response)
        }
    }
}
