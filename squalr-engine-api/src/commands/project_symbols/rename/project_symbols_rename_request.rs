use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::project_symbols::rename::project_symbols_rename_response::ProjectSymbolsRenameResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsRenameRequest {
    #[structopt(short = "k", long = "key")]
    pub symbol_locator_key: String,

    #[structopt(short = "n", long = "name")]
    pub display_name: String,
}

impl UnprivilegedCommandRequest for ProjectSymbolsRenameRequest {
    type ResponseType = ProjectSymbolsRenameResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::Rename {
            project_symbols_rename_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsRenameResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_rename_response: ProjectSymbolsRenameResponse) -> Self {
        ProjectSymbolsResponse::Rename {
            project_symbols_rename_response,
        }
    }
}
