use crate::commands::project_symbols::create::project_symbols_create_response::ProjectSymbolsCreateResponse;
use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsCreateRequest {
    pub display_name: String,
    pub struct_layout_id: String,

    #[serde(default)]
    pub address: Option<u64>,

    #[serde(default)]
    pub module_name: Option<String>,

    #[serde(default)]
    pub offset: Option<u64>,

    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl UnprivilegedCommandRequest for ProjectSymbolsCreateRequest {
    type ResponseType = ProjectSymbolsCreateResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::Create {
            project_symbols_create_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsCreateResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_create_response: ProjectSymbolsCreateResponse) -> Self {
        ProjectSymbolsResponse::Create {
            project_symbols_create_response,
        }
    }
}
