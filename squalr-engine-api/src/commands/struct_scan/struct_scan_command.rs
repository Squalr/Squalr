use crate::commands::struct_scan::struct_scan_request::StructScanRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct StructScanCommand {
    #[structopt(flatten)]
    pub struct_scan_request: StructScanRequest,
}
