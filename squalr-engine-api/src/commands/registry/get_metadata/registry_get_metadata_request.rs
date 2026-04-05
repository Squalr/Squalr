use crate::commands::registry::get_metadata::registry_get_metadata_response::RegistryGetMetadataResponse;
use crate::commands::registry::registry_command::RegistryCommand;
use crate::commands::registry::registry_response::RegistryResponse;
use crate::commands::{privileged_command::PrivilegedCommand, privileged_command_request::PrivilegedCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct RegistryGetMetadataRequest {}

impl PrivilegedCommandRequest for RegistryGetMetadataRequest {
    type ResponseType = RegistryGetMetadataResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Registry(RegistryCommand::GetMetadata {
            registry_get_metadata_request: self.clone(),
        })
    }
}

impl From<RegistryGetMetadataResponse> for RegistryResponse {
    fn from(registry_get_metadata_response: RegistryGetMetadataResponse) -> Self {
        RegistryResponse::GetMetadata {
            registry_get_metadata_response,
        }
    }
}
