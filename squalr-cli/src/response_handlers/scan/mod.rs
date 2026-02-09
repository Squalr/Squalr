pub mod handler_scan_collect_values_response;
pub mod handler_scan_executor_response;
pub mod handler_scan_new_response;
pub mod handler_scan_reset_response;

use crate::response_handlers::scan::handler_scan_collect_values_response::handle_scan_collect_values_response;
use crate::response_handlers::scan::handler_scan_executor_response::handle_scan_execute_response;
use crate::response_handlers::scan::handler_scan_new_response::handle_scan_new_response;
use crate::response_handlers::scan::handler_scan_reset_response::handle_scan_reset_response;
use squalr_engine_api::commands::scan::scan_response::ScanResponse;

pub fn handle_scan_response(cmd: ScanResponse) {
    match cmd {
        ScanResponse::Reset { .. } => handle_scan_reset_response(cmd),
        ScanResponse::New { .. } => handle_scan_new_response(cmd),
        ScanResponse::CollectValues { .. } => handle_scan_collect_values_response(cmd),
        ScanResponse::ElementScan { .. } => handle_scan_execute_response(cmd),
    }
}
