use crate::commands::registry::get_metadata::registry_get_metadata_request::RegistryGetMetadataRequest;
use crate::commands::registry::set_project_symbols::registry_set_project_symbols_request::RegistrySetProjectSymbolsRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RegistryCommand {
    GetMetadata {
        registry_get_metadata_request: RegistryGetMetadataRequest,
    },
    SetProjectSymbols {
        registry_set_project_symbols_request: RegistrySetProjectSymbolsRequest,
    },
}
