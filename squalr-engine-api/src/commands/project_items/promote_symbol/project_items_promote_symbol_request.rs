use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::project_items::promote_symbol::project_items_promote_symbol_response::ProjectItemsPromoteSymbolResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsPromoteSymbolRequest {
    #[structopt(short = "p", long = "project-item-path", parse(from_os_str))]
    pub project_item_paths: Vec<PathBuf>,

    #[serde(default)]
    #[structopt(long = "overwrite-conflicting-symbols")]
    pub overwrite_conflicting_symbols: bool,
}

impl UnprivilegedCommandRequest for ProjectItemsPromoteSymbolRequest {
    type ResponseType = ProjectItemsPromoteSymbolResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::PromoteSymbol {
            project_items_promote_symbol_request: self.clone(),
        })
    }
}

impl From<ProjectItemsPromoteSymbolResponse> for ProjectItemsResponse {
    fn from(project_items_promote_symbol_response: ProjectItemsPromoteSymbolResponse) -> Self {
        ProjectItemsResponse::PromoteSymbol {
            project_items_promote_symbol_response,
        }
    }
}
