use crate::commands::registry::get_metadata::registry_get_metadata_request::RegistryGetMetadataRequest;
use crate::commands::registry::set_project_symbols::registry_set_project_symbols_request::RegistrySetProjectSymbolsRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum RegistryCommand {
    GetMetadata {
        #[structopt(flatten)]
        registry_get_metadata_request: RegistryGetMetadataRequest,
    },
    SetProjectSymbols {
        #[structopt(skip)]
        registry_set_project_symbols_request: RegistrySetProjectSymbolsRequest,
    },
}
