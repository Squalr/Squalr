pub mod handler_scan_collect_values_response;
pub mod handler_scan_hybrid_response;
pub mod handler_scan_manual_response;
pub mod handler_scan_new_response;

use crate::response_handlers::scan::handler_scan_collect_values_response::handle_scan_collect_values_response;
use crate::response_handlers::scan::handler_scan_hybrid_response::handle_scan_hybrid_response;
use crate::response_handlers::scan::handler_scan_manual_response::handle_scan_manual_response;
use crate::response_handlers::scan::handler_scan_new_response::handle_scan_new_response;
use squalr_engine::command_executors::scan::scan_response::ScanResponse;

pub fn handle_scan_response(cmd: ScanResponse) {
    match cmd {
        ScanResponse::New { .. } => handle_scan_new_response(cmd),
        ScanResponse::CollectValues { .. } => handle_scan_collect_values_response(cmd),
        ScanResponse::Manual { .. } => handle_scan_manual_response(cmd),
        ScanResponse::Hybrid { .. } => handle_scan_hybrid_response(cmd),
    }
}
