use crate::commands::project::project_response::ProjectResponse;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnprivilegedCommandResponse {
    Project(ProjectResponse),
    ProjectItems(ProjectItemsResponse),
}

pub trait TypedUnprivilegedCommandResponse: Sized {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse;
    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse>;
}
