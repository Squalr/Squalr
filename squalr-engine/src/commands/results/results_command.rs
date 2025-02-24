use std::sync::Arc;

use crate::commands::engine_request::EngineRequest;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::{engine_response::EngineResponse, results::list::results_list_request::ResultsListRequest};
use crate::engine_execution_context::EngineExecutionContext;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ResultsCommand {
    /// Collect values and scan in the same parallel thread pool.
    List {
        #[structopt(flatten)]
        results_list_request: ResultsListRequest,
    },
}

impl ResultsCommand {
    pub fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> EngineResponse {
        match self {
            ResultsCommand::List { results_list_request } => results_list_request
                .execute(execution_context)
                .to_engine_response(),
        }
    }
}
