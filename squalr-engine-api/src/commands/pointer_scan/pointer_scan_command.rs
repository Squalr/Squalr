use crate::commands::pointer_scan::expand::pointer_scan_expand_request::PointerScanExpandRequest;
use crate::commands::pointer_scan::reset::pointer_scan_reset_request::PointerScanResetRequest;
use crate::commands::pointer_scan::start::pointer_scan_start_request::PointerScanStartRequest;
use crate::commands::pointer_scan::summary::pointer_scan_summary_request::PointerScanSummaryRequest;
use crate::commands::pointer_scan::validate::pointer_scan_validate_request::PointerScanValidateRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PointerScanCommand {
    Reset {
        pointer_scan_reset_request: PointerScanResetRequest,
    },
    Start {
        pointer_scan_start_request: PointerScanStartRequest,
    },
    Summary {
        pointer_scan_summary_request: PointerScanSummaryRequest,
    },
    Expand {
        pointer_scan_expand_request: PointerScanExpandRequest,
    },
    Validate {
        pointer_scan_validate_request: PointerScanValidateRequest,
    },
}
