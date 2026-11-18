use crate::commands::project::project_response::ProjectResponse;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnprivilegedCommandResponse {
    Failed { message: String },
    Project(ProjectResponse),
    ProjectItems(ProjectItemsResponse),
    ProjectSymbols(ProjectSymbolsResponse),
}

pub trait TypedUnprivilegedCommandResponse: Sized {
    fn to_engine_response(&self) -> UnprivilegedCommandResponse;
    fn from_engine_response(response: UnprivilegedCommandResponse) -> Result<Self, UnprivilegedCommandResponse>;
}

impl UnprivilegedCommandResponse {
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::Failed { message } => Some(message),
            Self::Project(reply) => reply.failure_message(),
            Self::ProjectItems(reply) => reply.failure_message(),
            Self::ProjectSymbols(reply) => reply.failure_message(),
        }
    }
}
