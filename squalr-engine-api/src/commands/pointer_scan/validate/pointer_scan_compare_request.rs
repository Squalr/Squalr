use super::pointer_scan_validate_request::PointerScanValidateRequest;
use super::pointer_scan_validate_response::PointerScanValidateResponse;
use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

/// Compare a second target without replacing the primary graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PointerScanCompareRequest(pub PointerScanValidateRequest);

impl PrivilegedCommandRequest for PointerScanCompareRequest {
    type ResponseType = PointerScanValidateResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::PointerScan(PointerScanCommand::Compare {
            pointer_scan_compare_request: self.clone(),
        })
    }
}
