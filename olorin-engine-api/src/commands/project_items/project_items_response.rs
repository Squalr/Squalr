use crate::commands::project_items::{
    activate::project_items_activate_response::ProjectItemsActivateResponse, list::project_items_list_response::ProjectItemsListResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectItemsResponse {
    Activate {
        project_items_activate_response: ProjectItemsActivateResponse,
    },
    List {
        project_items_list_response: ProjectItemsListResponse,
    },
}
