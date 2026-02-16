use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::project_items::rename::project_items_rename_response::ProjectItemsRenameResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsRenameRequest {
    #[structopt(short = "p", long)]
    pub project_item_path: PathBuf,

    #[structopt(short = "n", long)]
    pub project_item_name: String,
}

impl UnprivilegedCommandRequest for ProjectItemsRenameRequest {
    type ResponseType = ProjectItemsRenameResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Rename {
            project_items_rename_request: self.clone(),
        })
    }
}

impl From<ProjectItemsRenameResponse> for ProjectItemsResponse {
    fn from(project_items_rename_response: ProjectItemsRenameResponse) -> Self {
        ProjectItemsResponse::Rename { project_items_rename_response }
    }
}
