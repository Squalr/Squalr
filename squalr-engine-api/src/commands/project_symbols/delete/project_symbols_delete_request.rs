use crate::commands::project_symbols::delete::project_symbols_delete_response::ProjectSymbolsDeleteResponse;
use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectSymbolsDeleteModuleRangeMode {
    #[default]
    ShiftLeft,
    ReplaceWithU8,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectSymbolsDeleteModuleRange {
    pub module_name: String,
    pub offset: u64,
    pub length: u64,
    #[serde(default)]
    pub mode: ProjectSymbolsDeleteModuleRangeMode,
}

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsDeleteRequest {
    #[structopt(short = "k", long = "key")]
    pub symbol_locator_keys: Vec<String>,

    #[structopt(short = "m", long = "module")]
    pub module_names: Vec<String>,

    #[serde(default)]
    #[structopt(skip)]
    pub module_ranges: Vec<ProjectSymbolsDeleteModuleRange>,
}

impl UnprivilegedCommandRequest for ProjectSymbolsDeleteRequest {
    type ResponseType = ProjectSymbolsDeleteResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::Delete {
            project_symbols_delete_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsDeleteResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_delete_response: ProjectSymbolsDeleteResponse) -> Self {
        ProjectSymbolsResponse::Delete {
            project_symbols_delete_response,
        }
    }
}
