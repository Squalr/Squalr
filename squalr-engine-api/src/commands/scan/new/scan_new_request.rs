use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::scan::new::scan_new_response::ScanNewResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::commands::{privileged_command::PrivilegedCommand, scan::scan_command::ScanCommand};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanNewRequest {}

impl PrivilegedCommandRequest for ScanNewRequest {
    type ResponseType = ScanNewResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Scan(ScanCommand::New {
            scan_new_request: self.clone(),
        })
    }
}

impl From<ScanNewResponse> for ScanResponse {
    fn from(scan_new_response: ScanNewResponse) -> Self {
        ScanResponse::New { scan_new_response }
    }
}
