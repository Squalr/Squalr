use crate::commands::pointer_scan::pointer_scan_request::PointerScanRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct PointerScanCommand {
    #[structopt(flatten)]
    pub pointer_scan_request: PointerScanRequest,
}
