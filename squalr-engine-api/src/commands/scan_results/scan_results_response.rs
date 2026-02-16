use crate::commands::scan_results::delete::scan_results_delete_response::ScanResultsDeleteResponse;
use crate::commands::scan_results::freeze::scan_results_freeze_response::ScanResultsFreezeResponse;
use crate::commands::scan_results::list::scan_results_list_response::ScanResultsListResponse;
use crate::commands::scan_results::query::scan_results_query_response::ScanResultsQueryResponse;
use crate::commands::scan_results::refresh::scan_results_refresh_response::ScanResultsRefreshResponse;
use crate::commands::scan_results::set_property::scan_results_set_property_response::ScanResultsSetPropertyResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanResultsResponse {
    List {
        scan_results_list_response: ScanResultsListResponse,
    },
    Query {
        scan_results_query_response: ScanResultsQueryResponse,
    },
    Refresh {
        scan_results_refresh_response: ScanResultsRefreshResponse,
    },
    Freeze {
        scan_results_freeze_response: ScanResultsFreezeResponse,
    },
    SetProperty {
        scan_results_set_property_response: ScanResultsSetPropertyResponse,
    },
    Delete {
        scan_results_delete_response: ScanResultsDeleteResponse,
    },
}
