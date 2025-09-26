use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan_results::query::scan_results_query_response::ScanResultsQueryResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// A request to fetch scan results without reading up-to-date values.
/// For fetching values, either use a ListRequest or pair this with a RefreshRequest.
#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsQueryRequest {
    #[structopt(short = "p", long)]
    pub page_index: u64,
}

impl EngineCommandRequest for ScanResultsQueryRequest {
    type ResponseType = ScanResultsQueryResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Results(ScanResultsCommand::Query {
            results_query_request: self.clone(),
        })
    }
}

impl From<ScanResultsQueryResponse> for ScanResultsResponse {
    fn from(scan_results_query_response: ScanResultsQueryResponse) -> Self {
        ScanResultsResponse::Query { scan_results_query_response }
    }
}
