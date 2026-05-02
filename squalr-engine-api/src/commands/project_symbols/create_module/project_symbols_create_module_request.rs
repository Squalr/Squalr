use crate::commands::project_symbols::create_module::project_symbols_create_module_response::ProjectSymbolsCreateModuleResponse;
use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsCreateModuleRequest {
    #[structopt(short = "m", long = "module")]
    pub module_name: String,

    #[structopt(short = "s", long = "size")]
    pub size: u64,
}

impl UnprivilegedCommandRequest for ProjectSymbolsCreateModuleRequest {
    type ResponseType = ProjectSymbolsCreateModuleResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::CreateModule {
            project_symbols_create_module_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsCreateModuleResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_create_module_response: ProjectSymbolsCreateModuleResponse) -> Self {
        ProjectSymbolsResponse::CreateModule {
            project_symbols_create_module_response,
        }
    }
}
