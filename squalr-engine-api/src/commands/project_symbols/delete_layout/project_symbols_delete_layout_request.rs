use crate::commands::project_symbols::delete_layout::project_symbols_delete_layout_response::ProjectSymbolsDeleteLayoutResponse;
use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsDeleteLayoutRequest {
    #[structopt(short = "i", long = "id")]
    pub struct_layout_id: String,

    #[serde(default = "default_replacement_data_type_id")]
    #[structopt(long = "replacement-type", default_value = "u8")]
    pub replacement_data_type_id: String,
}

impl ProjectSymbolsDeleteLayoutRequest {
    pub fn new(struct_layout_id: impl Into<String>) -> Self {
        Self {
            struct_layout_id: struct_layout_id.into(),
            replacement_data_type_id: default_replacement_data_type_id(),
        }
    }
}

impl UnprivilegedCommandRequest for ProjectSymbolsDeleteLayoutRequest {
    type ResponseType = ProjectSymbolsDeleteLayoutResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::DeleteLayout {
            project_symbols_delete_layout_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsDeleteLayoutResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_delete_layout_response: ProjectSymbolsDeleteLayoutResponse) -> Self {
        ProjectSymbolsResponse::DeleteLayout {
            project_symbols_delete_layout_response,
        }
    }
}

fn default_replacement_data_type_id() -> String {
    String::from("u8")
}
