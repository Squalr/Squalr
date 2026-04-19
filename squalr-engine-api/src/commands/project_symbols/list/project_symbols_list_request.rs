use crate::commands::project_symbols::list::project_symbols_list_response::ProjectSymbolsListResponse;
use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsListRequest {}

impl UnprivilegedCommandRequest for ProjectSymbolsListRequest {
    type ResponseType = ProjectSymbolsListResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::List {
            project_symbols_list_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsListResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_list_response: ProjectSymbolsListResponse) -> Self {
        ProjectSymbolsResponse::List { project_symbols_list_response }
    }
}
