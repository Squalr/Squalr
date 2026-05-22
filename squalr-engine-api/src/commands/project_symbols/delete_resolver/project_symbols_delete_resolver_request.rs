use crate::commands::project_symbols::delete_resolver::project_symbols_delete_resolver_response::ProjectSymbolsDeleteResolverResponse;
use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsDeleteResolverRequest {
    pub resolver_id: String,
}

impl ProjectSymbolsDeleteResolverRequest {
    pub fn new(resolver_id: impl Into<String>) -> Self {
        Self {
            resolver_id: resolver_id.into(),
        }
    }
}

impl UnprivilegedCommandRequest for ProjectSymbolsDeleteResolverRequest {
    type ResponseType = ProjectSymbolsDeleteResolverResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::DeleteResolver {
            project_symbols_delete_resolver_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsDeleteResolverResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_delete_resolver_response: ProjectSymbolsDeleteResolverResponse) -> Self {
        ProjectSymbolsResponse::DeleteResolver {
            project_symbols_delete_resolver_response,
        }
    }
}
