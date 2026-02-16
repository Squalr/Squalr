use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::project_items::reorder::project_items_reorder_response::ProjectItemsReorderResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsReorderRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,
}

impl UnprivilegedCommandRequest for ProjectItemsReorderRequest {
    type ResponseType = ProjectItemsReorderResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Reorder {
            project_items_reorder_request: self.clone(),
        })
    }
}

impl From<ProjectItemsReorderResponse> for ProjectItemsResponse {
    fn from(project_items_reorder_response: ProjectItemsReorderResponse) -> Self {
        ProjectItemsResponse::Reorder {
            project_items_reorder_response,
        }
    }
}
