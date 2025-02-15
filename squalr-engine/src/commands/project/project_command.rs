use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::project::project_request::ProjectRequest;
use crate::commands::{engine_response::EngineResponse, project::list::project_list_request::ProjectListRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectCommand {
    /// List all projects
    List {
        #[structopt(flatten)]
        project_list_request: ProjectListRequest,
    },
}

impl ProjectCommand {
    pub fn execute(&self) -> EngineResponse {
        match self {
            ProjectCommand::List { project_list_request } => project_list_request.execute().to_engine_response(),
        }
    }
}
