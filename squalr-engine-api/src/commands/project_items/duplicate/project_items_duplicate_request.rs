use crate::commands::project_items::duplicate::project_items_duplicate_response::ProjectItemsDuplicateResponse;
use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsDuplicateRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,

    #[structopt(short = "t", long)]
    pub target_directory_path: PathBuf,
}

impl UnprivilegedCommandRequest for ProjectItemsDuplicateRequest {
    type ResponseType = ProjectItemsDuplicateResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Duplicate {
            project_items_duplicate_request: self.clone(),
        })
    }
}

impl From<ProjectItemsDuplicateResponse> for ProjectItemsResponse {
    fn from(project_items_duplicate_response: ProjectItemsDuplicateResponse) -> Self {
        ProjectItemsResponse::Duplicate {
            project_items_duplicate_response,
        }
    }
}
