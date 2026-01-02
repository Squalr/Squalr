use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::{project_items::list::project_items_list_response::ProjectItemsListResponse, unprivileged_command_request::UnprivilegedCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsListRequest {}

impl UnprivilegedCommandRequest for ProjectItemsListRequest {
    type ResponseType = ProjectItemsListResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::List {
            project_items_list_request: self.clone(),
        })
    }
}

impl From<ProjectItemsListResponse> for ProjectItemsResponse {
    fn from(project_items_list_response: ProjectItemsListResponse) -> Self {
        ProjectItemsResponse::List { project_items_list_response }
    }
}
