use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::project_items::strip_symbol::project_items_strip_symbol_response::ProjectItemsStripSymbolResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProjectItemsStripSymbolRequest {
    pub project_item_paths: Vec<PathBuf>,
}

impl UnprivilegedCommandRequest for ProjectItemsStripSymbolRequest {
    type ResponseType = ProjectItemsStripSymbolResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::StripSymbol {
            project_items_strip_symbol_request: self.clone(),
        })
    }
}

impl From<ProjectItemsStripSymbolResponse> for ProjectItemsResponse {
    fn from(project_items_strip_symbol_response: ProjectItemsStripSymbolResponse) -> Self {
        ProjectItemsResponse::StripSymbol {
            project_items_strip_symbol_response,
        }
    }
}
