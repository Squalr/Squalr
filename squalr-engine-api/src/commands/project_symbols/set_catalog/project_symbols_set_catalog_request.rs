use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::project_symbols::set_catalog::project_symbols_set_catalog_response::ProjectSymbolsSetCatalogResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsSetCatalogRequest {
    #[serde(default)]
    #[structopt(skip)]
    pub project_symbol_catalog: Option<ProjectSymbolCatalog>,
}

impl ProjectSymbolsSetCatalogRequest {
    pub fn new(project_symbol_catalog: ProjectSymbolCatalog) -> Self {
        Self {
            project_symbol_catalog: Some(project_symbol_catalog),
        }
    }
}

impl UnprivilegedCommandRequest for ProjectSymbolsSetCatalogRequest {
    type ResponseType = ProjectSymbolsSetCatalogResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::SetCatalog {
            project_symbols_set_catalog_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsSetCatalogResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_set_catalog_response: ProjectSymbolsSetCatalogResponse) -> Self {
        ProjectSymbolsResponse::SetCatalog {
            project_symbols_set_catalog_response,
        }
    }
}
