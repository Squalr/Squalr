use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::{
    project_items::activate::project_items_activate_response::ProjectItemsActivateResponse, unprivileged_command_request::UnprivilegedCommandRequest,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsActivateRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<String>,
    #[structopt(short = "a", long)]
    pub is_activated: bool,
}

impl UnprivilegedCommandRequest for ProjectItemsActivateRequest {
    type ResponseType = ProjectItemsActivateResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Activate {
            project_items_activate_request: self.clone(),
        })
    }
}

impl From<ProjectItemsActivateResponse> for ProjectItemsResponse {
    fn from(project_items_activate_response: ProjectItemsActivateResponse) -> Self {
        ProjectItemsResponse::Activate {
            project_items_activate_response,
        }
    }
}
