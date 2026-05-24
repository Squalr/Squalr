use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::project_symbols::rename_module::project_symbols_rename_module_response::ProjectSymbolsRenameModuleResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsRenameModuleRequest {
    pub module_name: String,
    pub new_module_name: String,
}

impl UnprivilegedCommandRequest for ProjectSymbolsRenameModuleRequest {
    type ResponseType = ProjectSymbolsRenameModuleResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::RenameModule {
            project_symbols_rename_module_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsRenameModuleResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_rename_module_response: ProjectSymbolsRenameModuleResponse) -> Self {
        ProjectSymbolsResponse::RenameModule {
            project_symbols_rename_module_response,
        }
    }
}
