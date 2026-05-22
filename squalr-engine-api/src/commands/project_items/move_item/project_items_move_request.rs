use crate::commands::project_items::move_item::project_items_move_response::ProjectItemsMoveResponse;
use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItemsMoveRequest {
    pub project_item_paths: Vec<PathBuf>,
    pub target_directory_path: PathBuf,
}

impl UnprivilegedCommandRequest for ProjectItemsMoveRequest {
    type ResponseType = ProjectItemsMoveResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Move {
            project_items_move_request: self.clone(),
        })
    }
}

impl From<ProjectItemsMoveResponse> for ProjectItemsResponse {
    fn from(project_items_move_response: ProjectItemsMoveResponse) -> Self {
        ProjectItemsResponse::Move { project_items_move_response }
    }
}
