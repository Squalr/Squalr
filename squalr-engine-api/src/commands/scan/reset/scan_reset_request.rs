use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::scan::reset::scan_reset_response::ScanResetResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::commands::{privileged_command::PrivilegedCommand, scan::scan_command::ScanCommand};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResetRequest {}

impl PrivilegedCommandRequest for ScanResetRequest {
    type ResponseType = ScanResetResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Scan(ScanCommand::Reset {
            scan_reset_request: self.clone(),
        })
    }
}

impl From<ScanResetResponse> for ScanResponse {
    fn from(scan_reset_response: ScanResetResponse) -> Self {
        ScanResponse::Reset { scan_reset_response }
    }
}
