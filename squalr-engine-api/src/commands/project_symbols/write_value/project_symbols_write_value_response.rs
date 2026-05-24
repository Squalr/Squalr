use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolsWriteValueResponse {
    pub success: bool,
    pub error: Option<String>,
}

impl TypedUnprivilegedCommandResponse for ProjectSymbolsWriteValueResponse {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse {
        UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::WriteValue {
            project_symbols_write_value_response: self.clone(),
        })
    }

    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse> {
        if let UnprivilegedCommandResponse::ProjectSymbols(ProjectSymbolsResponse::WriteValue {
            project_symbols_write_value_response,
        }) = response
        {
            Ok(project_symbols_write_value_response)
        } else {
            Err(response)
        }
    }
}
