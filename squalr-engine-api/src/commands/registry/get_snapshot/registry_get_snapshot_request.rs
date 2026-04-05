use crate::commands::registry::get_snapshot::registry_get_snapshot_response::RegistryGetSnapshotResponse;
use crate::commands::registry::registry_command::RegistryCommand;
use crate::commands::registry::registry_response::RegistryResponse;
use crate::commands::{privileged_command::PrivilegedCommand, privileged_command_request::PrivilegedCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct RegistryGetSnapshotRequest {}

impl PrivilegedCommandRequest for RegistryGetSnapshotRequest {
    type ResponseType = RegistryGetSnapshotResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Registry(RegistryCommand::GetSnapshot {
            registry_get_snapshot_request: self.clone(),
        })
    }
}

impl From<RegistryGetSnapshotResponse> for RegistryResponse {
    fn from(registry_get_snapshot_response: RegistryGetSnapshotResponse) -> Self {
        RegistryResponse::GetSnapshot {
            registry_get_snapshot_response,
        }
    }
}
