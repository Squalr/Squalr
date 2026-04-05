use crate::{commands::privileged_command_response::PrivilegedCommandResponse, registries::symbols::registry_metadata::RegistryMetadata};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivilegedCommandResult {
    privileged_command_response: PrivilegedCommandResponse,
    registry_metadata: Option<RegistryMetadata>,
}

impl PrivilegedCommandResult {
    pub fn new(
        privileged_command_response: PrivilegedCommandResponse,
        registry_metadata: Option<RegistryMetadata>,
    ) -> Self {
        Self {
            privileged_command_response,
            registry_metadata,
        }
    }

    pub fn get_privileged_command_response(&self) -> &PrivilegedCommandResponse {
        &self.privileged_command_response
    }

    pub fn into_privileged_command_response(self) -> PrivilegedCommandResponse {
        self.privileged_command_response
    }

    pub fn get_registry_metadata(&self) -> Option<&RegistryMetadata> {
        self.registry_metadata.as_ref()
    }
}
