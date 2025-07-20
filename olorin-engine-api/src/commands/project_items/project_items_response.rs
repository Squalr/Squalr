use crate::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectItemsResponse {
    Activate {
        project_items_activate_response: ProjectItemsActivateResponse,
    },
}
