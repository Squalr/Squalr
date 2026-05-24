use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::commands::registry::registry_response::RegistryResponse;
use crate::registries::symbols::privileged_registry_catalog::PrivilegedRegistryCatalog;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RegistryGetMetadataResponse {
    pub privileged_registry_catalog: PrivilegedRegistryCatalog,
}

impl TypedPrivilegedCommandResponse for RegistryGetMetadataResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Registry(RegistryResponse::GetMetadata {
            registry_get_metadata_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Registry(RegistryResponse::GetMetadata {
            registry_get_metadata_response,
        }) = response
        {
            Ok(registry_get_metadata_response)
        } else {
            Err(response)
        }
    }
}
