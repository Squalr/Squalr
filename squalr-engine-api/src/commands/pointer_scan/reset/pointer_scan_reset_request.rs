use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::pointer_scan::reset::pointer_scan_reset_response::PointerScanResetResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct PointerScanResetRequest {}

impl PrivilegedCommandRequest for PointerScanResetRequest {
    type ResponseType = PointerScanResetResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::PointerScan(PointerScanCommand::Reset {
            pointer_scan_reset_request: self.clone(),
        })
    }
}
