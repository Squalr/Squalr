use crate::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use crate::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use crate::commands::scan_results::list::scan_results_list_request::ScanResultsListRequest;
use crate::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use crate::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use crate::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanResultsCommand {
    /// Query scan results and fetch their results in a single request.
    List { results_list_request: ScanResultsListRequest },
    /// Query scan results without fetching their values.
    Query { results_query_request: ScanResultsQueryRequest },
    /// Uses the results of a Query operation to fetch the latest values for scan results.
    Refresh { results_refresh_request: ScanResultsRefreshRequest },
    /// Freezes a specified set of scan result addresses to their current value.
    Freeze { results_freeze_request: ScanResultsFreezeRequest },
    /// Sets a property on a specified set of scan results.
    SetProperty {
        results_set_property_request: ScanResultsSetPropertyRequest,
    },
    /// Deletes a specified set of scan results.
    Delete { results_delete_request: ScanResultsDeleteRequest },
}
