use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::commands::registry::registry_response::RegistryResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RegistrySetProjectSymbolsResponse {
    pub success: bool,
}

impl TypedPrivilegedCommandResponse for RegistrySetProjectSymbolsResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Registry(RegistryResponse::SetProjectSymbols {
            registry_set_project_symbols_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Registry(RegistryResponse::SetProjectSymbols {
            registry_set_project_symbols_response,
        }) = response
        {
            Ok(registry_set_project_symbols_response)
        } else {
            Err(response)
        }
    }
}
