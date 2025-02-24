use std::sync::Arc;

use crate::commands::engine_request::EngineRequest;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::{engine_response::EngineResponse, scan_results::list::scan_results_list_request::ScanResultsListRequest};
use crate::engine_execution_context::EngineExecutionContext;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ScanResultsCommand {
    /// Collect values and scan in the same parallel thread pool.
    List {
        #[structopt(flatten)]
        results_list_request: ScanResultsListRequest,
    },
}

impl ScanResultsCommand {
    pub fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> EngineResponse {
        match self {
            ScanResultsCommand::List { results_list_request } => results_list_request
                .execute(execution_context)
                .to_engine_response(),
        }
    }
}
