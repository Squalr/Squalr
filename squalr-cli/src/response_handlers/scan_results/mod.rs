pub mod handler_scan_results_list_response;

use crate::response_handlers::scan_results::handler_scan_results_list_response::handle_scan_results_list_response;
use squalr_engine_api::commands::scan_results::scan_results_response::ScanResultsResponse;

pub fn handle_scan_results_response(cmd: ScanResultsResponse) {
    match cmd {
        ScanResultsResponse::List { scan_results_list_response } => handle_scan_results_list_response(scan_results_list_response),
    }
}
