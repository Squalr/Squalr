use crate::commands::project_items::delete::project_items_delete_response::ProjectItemsDeleteResponse;
use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsDeleteRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,
}

impl UnprivilegedCommandRequest for ProjectItemsDeleteRequest {
    type ResponseType = ProjectItemsDeleteResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Delete {
            project_items_delete_request: self.clone(),
        })
    }
}

impl From<ProjectItemsDeleteResponse> for ProjectItemsResponse {
    fn from(project_items_delete_response: ProjectItemsDeleteResponse) -> Self {
        ProjectItemsResponse::Delete { project_items_delete_response }
    }
}
