use crate::commands::registry::get_metadata::registry_get_metadata_response::RegistryGetMetadataResponse;
use crate::commands::registry::set_project_symbols::registry_set_project_symbols_response::RegistrySetProjectSymbolsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RegistryResponse {
    GetMetadata {
        registry_get_metadata_response: RegistryGetMetadataResponse,
    },
    SetProjectSymbols {
        registry_set_project_symbols_response: RegistrySetProjectSymbolsResponse,
    },
}

impl RegistryResponse {
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::SetProjectSymbols {
                registry_set_project_symbols_response: reply,
            } if !reply.success => Some("The command failed."),
            _ => None,
        }
    }
}
