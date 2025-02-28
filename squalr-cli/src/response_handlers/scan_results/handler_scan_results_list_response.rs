use squalr_engine_api::commands::scan_results::list::scan_results_list_response::ScanResultsListResponse;

pub fn handle_scan_results_list_response(results_list_response: ScanResultsListResponse) {
    for scan_result in results_list_response.scan_results {
        log::info!("{:?}", scan_result);
    }
}
