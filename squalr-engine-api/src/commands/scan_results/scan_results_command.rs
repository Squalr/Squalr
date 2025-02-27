use crate::commands::scan_results::list::scan_results_list_request::ScanResultsListRequest;
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
