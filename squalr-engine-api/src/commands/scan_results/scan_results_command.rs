use crate::commands::scan_results::list::scan_results_list_request::ScanResultsListRequest;
use crate::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use crate::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ScanResultsCommand {
    /// Query scan results and fetch their results in a single request.
    List {
        #[structopt(flatten)]
        results_list_request: ScanResultsListRequest,
    },
    /// Query scan results without fetching their values.
    Query {
        #[structopt(flatten)]
        results_query_request: ScanResultsQueryRequest,
    },
    /// Uses the results of a Query operation to fetch the latest values for scan results.
    Refresh {
        #[structopt(flatten)]
        results_refresh_request: ScanResultsRefreshRequest,
    },
}
