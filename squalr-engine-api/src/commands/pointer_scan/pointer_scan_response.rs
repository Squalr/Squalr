use crate::commands::pointer_scan::expand::pointer_scan_expand_response::PointerScanExpandResponse;
use crate::commands::pointer_scan::start::pointer_scan_start_response::PointerScanStartResponse;
use crate::commands::pointer_scan::summary::pointer_scan_summary_response::PointerScanSummaryResponse;
use crate::commands::pointer_scan::validate::pointer_scan_validate_response::PointerScanValidateResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PointerScanResponse {
    Start {
        pointer_scan_start_response: PointerScanStartResponse,
    },
    Summary {
        pointer_scan_summary_response: PointerScanSummaryResponse,
    },
    Expand {
        pointer_scan_expand_response: PointerScanExpandResponse,
    },
    Validate {
        pointer_scan_validate_response: PointerScanValidateResponse,
    },
}
