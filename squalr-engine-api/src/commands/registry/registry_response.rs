use crate::commands::registry::get_snapshot::registry_get_snapshot_response::RegistryGetSnapshotResponse;
use crate::commands::registry::set_project_symbols::registry_set_project_symbols_response::RegistrySetProjectSymbolsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RegistryResponse {
    GetSnapshot {
        registry_get_snapshot_response: RegistryGetSnapshotResponse,
    },
    SetProjectSymbols {
        registry_set_project_symbols_response: RegistrySetProjectSymbolsResponse,
    },
}
