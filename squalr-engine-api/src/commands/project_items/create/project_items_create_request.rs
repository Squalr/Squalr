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

    #[serde(default)]
    #[structopt(long)]
    pub is_directory: bool,

    #[serde(default)]
    #[structopt(skip)]
    pub address: Option<u64>,

    #[serde(default)]
    #[structopt(skip)]
    pub module_name: Option<String>,

    #[serde(default)]
    #[structopt(skip)]
    pub data_type_id: Option<String>,
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
