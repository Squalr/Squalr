use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::project_symbols::update::project_symbols_update_response::ProjectSymbolsUpdateResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsUpdateRequest {
    pub symbol_locator_key: String,
    pub display_name: Option<String>,
    pub struct_layout_id: Option<String>,
}

impl UnprivilegedCommandRequest for ProjectSymbolsUpdateRequest {
    type ResponseType = ProjectSymbolsUpdateResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::Update {
            project_symbols_update_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsUpdateResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_update_response: ProjectSymbolsUpdateResponse) -> Self {
        ProjectSymbolsResponse::Update {
            project_symbols_update_response,
        }
    }
}
