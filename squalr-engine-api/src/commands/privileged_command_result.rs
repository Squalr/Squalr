use crate::{commands::privileged_command_response::PrivilegedCommandResponse, registries::symbols::privileged_registry_catalog::PrivilegedRegistryCatalog};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivilegedCommandResult {
    privileged_command_response: PrivilegedCommandResponse,
    privileged_registry_catalog: Option<PrivilegedRegistryCatalog>,
}

impl PrivilegedCommandResult {
    pub fn new(
        privileged_command_response: PrivilegedCommandResponse,
        privileged_registry_catalog: Option<PrivilegedRegistryCatalog>,
    ) -> Self {
        Self {
            privileged_command_response,
            privileged_registry_catalog,
        }
    }

    pub fn get_privileged_command_response(&self) -> &PrivilegedCommandResponse {
        &self.privileged_command_response
    }

    pub fn into_privileged_command_response(self) -> PrivilegedCommandResponse {
        self.privileged_command_response
    }

    pub fn get_privileged_registry_catalog(&self) -> Option<&PrivilegedRegistryCatalog> {
        self.privileged_registry_catalog.as_ref()
    }
}
