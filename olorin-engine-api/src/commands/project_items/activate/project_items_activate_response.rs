use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItemsActivateResponse {}

impl TypedEngineCommandResponse for ProjectItemsActivateResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::ProjectItems(ProjectItemsResponse::Activate {
            project_items_activate_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::ProjectItems(ProjectItemsResponse::Activate {
            project_items_activate_response,
        }) = response
        {
            Ok(project_items_activate_response)
        } else {
            Err(response)
        }
    }
}
