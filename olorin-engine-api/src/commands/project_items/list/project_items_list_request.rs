use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::project_items::list::project_items_list_response::ProjectItemsListResponse;
use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsListRequest {}

impl EngineCommandRequest for ProjectItemsListRequest {
    type ResponseType = ProjectItemsListResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::ProjectItems(ProjectItemsCommand::List {
            project_items_list_request: self.clone(),
        })
    }
}

impl From<ProjectItemsListResponse> for ProjectItemsResponse {
    fn from(project_items_list_response: ProjectItemsListResponse) -> Self {
        ProjectItemsResponse::List { project_items_list_response }
    }
}
