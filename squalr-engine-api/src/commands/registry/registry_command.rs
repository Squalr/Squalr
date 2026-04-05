use crate::commands::registry::get_snapshot::registry_get_snapshot_request::RegistryGetSnapshotRequest;
use crate::commands::registry::set_project_symbols::registry_set_project_symbols_request::RegistrySetProjectSymbolsRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum RegistryCommand {
    GetSnapshot {
        #[structopt(flatten)]
        registry_get_snapshot_request: RegistryGetSnapshotRequest,
    },
    SetProjectSymbols {
        #[structopt(skip)]
        registry_set_project_symbols_request: RegistrySetProjectSymbolsRequest,
    },
}
