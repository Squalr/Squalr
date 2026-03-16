use crate::commands::pointer_scan::expand::pointer_scan_expand_request::PointerScanExpandRequest;
use crate::commands::pointer_scan::start::pointer_scan_start_request::PointerScanStartRequest;
use crate::commands::pointer_scan::summary::pointer_scan_summary_request::PointerScanSummaryRequest;
use crate::commands::pointer_scan::validate::pointer_scan_validate_request::PointerScanValidateRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum PointerScanCommand {
    Start {
        #[structopt(flatten)]
        pointer_scan_start_request: PointerScanStartRequest,
    },
    Summary {
        #[structopt(flatten)]
        pointer_scan_summary_request: PointerScanSummaryRequest,
    },
    Expand {
        #[structopt(flatten)]
        pointer_scan_expand_request: PointerScanExpandRequest,
    },
    Validate {
        #[structopt(flatten)]
        pointer_scan_validate_request: PointerScanValidateRequest,
    },
}
