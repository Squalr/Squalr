use crate::{commands::privileged_command_response::PrivilegedCommandResponse, registries::symbols::symbol_registry_snapshot::RegistryMetadata};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivilegedCommandResult {
    privileged_command_response: PrivilegedCommandResponse,
    symbol_registry_snapshot: Option<RegistryMetadata>,
}

impl PrivilegedCommandResult {
    pub fn new(
        privileged_command_response: PrivilegedCommandResponse,
        symbol_registry_snapshot: Option<RegistryMetadata>,
    ) -> Self {
        Self {
            privileged_command_response,
            symbol_registry_snapshot,
        }
    }

    pub fn get_privileged_command_response(&self) -> &PrivilegedCommandResponse {
        &self.privileged_command_response
    }

    pub fn into_privileged_command_response(self) -> PrivilegedCommandResponse {
        self.privileged_command_response
    }

    pub fn get_symbol_registry_snapshot(&self) -> Option<&RegistryMetadata> {
        self.symbol_registry_snapshot.as_ref()
    }
}
