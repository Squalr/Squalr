use crate::commands::project_symbols::create::project_symbols_create_response::ProjectSymbolsCreateResponse;
use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsCreateRequest {
    #[structopt(short = "n", long = "name")]
    pub display_name: String,

    #[structopt(short = "t", long = "type")]
    pub struct_layout_id: String,

    #[serde(default)]
    #[structopt(short = "a", long = "address")]
    pub address: Option<u64>,

    #[serde(default)]
    #[structopt(short = "m", long = "module")]
    pub module_name: Option<String>,

    #[serde(default)]
    #[structopt(short = "o", long = "offset")]
    pub offset: Option<u64>,

    #[serde(default)]
    #[structopt(skip)]
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
