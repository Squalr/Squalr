use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsActivateRequest {
    #[structopt(short = "p", long)]
    pub project_item_ids: Vec<String>,
    #[structopt(short = "a", long)]
    pub is_activated: bool,
}

impl EngineCommandRequest for ProjectItemsActivateRequest {
    type ResponseType = ProjectItemsActivateResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::ProjectItems(ProjectItemsCommand::Activate {
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
