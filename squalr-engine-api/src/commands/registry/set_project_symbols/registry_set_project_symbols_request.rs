use crate::commands::registry::registry_command::RegistryCommand;
use crate::commands::registry::registry_response::RegistryResponse;
use crate::commands::registry::set_project_symbols::registry_set_project_symbols_response::RegistrySetProjectSymbolsResponse;
use crate::commands::{privileged_command::PrivilegedCommand, privileged_command_request::PrivilegedCommandRequest};
use crate::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct RegistrySetProjectSymbolsRequest {
    #[structopt(skip)]
    pub project_symbol_catalog: ProjectSymbolCatalog,
}

impl PrivilegedCommandRequest for RegistrySetProjectSymbolsRequest {
    type ResponseType = RegistrySetProjectSymbolsResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Registry(RegistryCommand::SetProjectSymbols {
            registry_set_project_symbols_request: self.clone(),
        })
    }
}

impl From<RegistrySetProjectSymbolsResponse> for RegistryResponse {
    fn from(registry_set_project_symbols_response: RegistrySetProjectSymbolsResponse) -> Self {
        RegistryResponse::SetProjectSymbols {
            registry_set_project_symbols_response,
        }
    }
}
