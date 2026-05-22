use crate::commands::project_items::create::project_items_create_response::ProjectItemsCreateResponse;
use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::structures::memory::pointer_chain_segment::PointerChainSegment;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItemsCreateRequest {
    pub parent_directory_path: PathBuf,
    pub project_item_name: String,

    #[serde(default)]
    pub is_directory: bool,

    #[serde(default)]
    pub address: Option<u64>,

    #[serde(default)]
    pub module_name: Option<String>,

    #[serde(default)]
    pub data_type_id: Option<String>,

    #[serde(default)]
    pub pointer_offsets: Option<Vec<PointerChainSegment>>,
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
