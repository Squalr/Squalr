use crate::commands::project_symbols::delete::project_symbols_delete_response::ProjectSymbolsDeleteResponse;
use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsDeleteRequest {
    #[structopt(short = "k", long = "key")]
    pub symbol_locator_keys: Vec<String>,

    #[structopt(short = "m", long = "module")]
    pub module_names: Vec<String>,
}

impl UnprivilegedCommandRequest for ProjectSymbolsDeleteRequest {
    type ResponseType = ProjectSymbolsDeleteResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::Delete {
            project_symbols_delete_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsDeleteResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_delete_response: ProjectSymbolsDeleteResponse) -> Self {
        ProjectSymbolsResponse::Delete {
            project_symbols_delete_response,
        }
    }
}
