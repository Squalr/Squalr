use crate::commands::engine_request::EngineRequest;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::{engine_response::EngineResponse, project::list::project_list_request::ProjectListRequest};
use crate::engine_execution_context::EngineExecutionContext;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
    pub fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> EngineResponse {
        match self {
            ProjectCommand::List { project_list_request } => project_list_request
                .execute(execution_context)
                .to_engine_response(),
        }
    }
}
