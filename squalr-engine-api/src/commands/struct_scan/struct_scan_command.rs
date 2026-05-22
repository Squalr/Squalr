use crate::commands::struct_scan::struct_scan_request::StructScanRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructScanCommand {
    pub struct_scan_request: StructScanRequest,
}
