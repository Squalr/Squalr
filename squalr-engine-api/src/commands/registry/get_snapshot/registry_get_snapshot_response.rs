use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::commands::registry::registry_response::RegistryResponse;
use crate::registries::symbols::symbol_registry_snapshot::SymbolRegistrySnapshot;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RegistryGetSnapshotResponse {
    pub symbol_registry_snapshot: SymbolRegistrySnapshot,
}

impl TypedPrivilegedCommandResponse for RegistryGetSnapshotResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Registry(RegistryResponse::GetSnapshot {
            registry_get_snapshot_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Registry(RegistryResponse::GetSnapshot {
            registry_get_snapshot_response,
        }) = response
        {
            Ok(registry_get_snapshot_response)
        } else {
            Err(response)
        }
    }
}
