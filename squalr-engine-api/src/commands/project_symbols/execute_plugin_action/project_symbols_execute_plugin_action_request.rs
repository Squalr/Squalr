use crate::commands::project_symbols::execute_plugin_action::project_symbols_execute_plugin_action_response::ProjectSymbolsExecutePluginActionResponse;
use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::plugins::symbol_tree::symbol_tree_action::SymbolTreeActionContext;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsExecutePluginActionRequest {
    pub plugin_id: String,
    pub action_id: String,
    pub context: SymbolTreeActionContext,
}

impl UnprivilegedCommandRequest for ProjectSymbolsExecutePluginActionRequest {
    type ResponseType = ProjectSymbolsExecutePluginActionResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::ExecutePluginAction {
            project_symbols_execute_plugin_action_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsExecutePluginActionResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_execute_plugin_action_response: ProjectSymbolsExecutePluginActionResponse) -> Self {
        ProjectSymbolsResponse::ExecutePluginAction {
            project_symbols_execute_plugin_action_response,
        }
    }
}
