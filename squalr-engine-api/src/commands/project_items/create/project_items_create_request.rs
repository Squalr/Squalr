use crate::commands::project_items::create::project_items_create_response::ProjectItemsCreateResponse;
use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsCreateRequest {
    #[structopt(short = "p", long)]
    pub parent_directory_path: PathBuf,

    #[structopt(short = "n", long)]
    pub project_item_name: String,

    #[structopt(short = "t", long, default_value = "directory")]
    pub project_item_type: String,
}

impl UnprivilegedCommandRequest for ProjectItemsCreateRequest {
    type ResponseType = ProjectItemsCreateResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Create {
            project_items_create_request: self.clone(),
        })
    }
}

impl From<ProjectItemsCreateResponse> for ProjectItemsResponse {
    fn from(project_items_create_response: ProjectItemsCreateResponse) -> Self {
        ProjectItemsResponse::Create { project_items_create_response }
    }
}
