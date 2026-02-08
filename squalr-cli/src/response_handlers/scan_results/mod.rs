pub mod handler_scan_results_list_response;

use crate::response_handlers::scan_results::handler_scan_results_list_response::handle_scan_results_list_response;
use squalr_engine_api::commands::scan_results::scan_results_response::ScanResultsResponse;

pub fn handle_scan_results_response(cmd: ScanResultsResponse) {
    match cmd {
        ScanResultsResponse::List { scan_results_list_response } => handle_scan_results_list_response(scan_results_list_response),
        ScanResultsResponse::Query { scan_results_query_response } => {
            log::debug!("Unhandled scan results query response: {:?}", scan_results_query_response);
        }
        ScanResultsResponse::Refresh { scan_results_refresh_response } => {
            log::debug!("Unhandled scan results refresh response: {:?}", scan_results_refresh_response);
        }
        ScanResultsResponse::AddToProject {
            scan_results_add_to_project_response,
        } => {
            log::debug!("Unhandled scan results add-to-project response: {:?}", scan_results_add_to_project_response);
        }
        ScanResultsResponse::Freeze { scan_results_freeze_response } => {
            log::debug!("Unhandled scan results freeze response: {:?}", scan_results_freeze_response);
        }
        ScanResultsResponse::SetProperty {
            scan_results_set_property_response,
        } => {
            log::debug!("Unhandled scan results set-property response: {:?}", scan_results_set_property_response);
        }
        ScanResultsResponse::Delete { scan_results_delete_response } => {
            log::debug!("Unhandled scan results delete response: {:?}", scan_results_delete_response);
        }
    }
}
