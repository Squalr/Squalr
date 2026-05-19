use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::project_symbols::write_value::project_symbols_write_value_response::ProjectSymbolsWriteValueResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsWriteValueRequest {
    #[structopt(long = "address")]
    pub address: u64,

    #[structopt(long = "module", default_value = "")]
    pub module_name: String,

    #[structopt(long = "type")]
    pub symbol_type_id: String,

    #[structopt(long = "container", default_value = "")]
    pub container_type: ContainerType,

    #[structopt(long = "field")]
    pub field_name: String,

    #[structopt(short = "v", long)]
    pub anonymous_value_string: AnonymousValueString,
}

impl UnprivilegedCommandRequest for ProjectSymbolsWriteValueRequest {
    type ResponseType = ProjectSymbolsWriteValueResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::WriteValue {
            project_symbols_write_value_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsWriteValueResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_write_value_response: ProjectSymbolsWriteValueResponse) -> Self {
        ProjectSymbolsResponse::WriteValue {
            project_symbols_write_value_response,
        }
    }
}
