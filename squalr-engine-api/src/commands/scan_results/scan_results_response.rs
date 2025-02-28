use crate::commands::scan_results::list::scan_results_list_response::ScanResultsListResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanResultsResponse {
    List { scan_results_list_response: ScanResultsListResponse },
}
