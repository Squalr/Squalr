use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::pointer_scan::validate::pointer_scan_validate_response::PointerScanValidateResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::structures::pointer_scans::pointer_scan_target_request::PointerScanTargetRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct PointerScanValidateRequest {
    #[structopt(short = "i", long)]
    pub session_id: u64,
    #[structopt(flatten)]
    pub target: PointerScanTargetRequest,
}

impl PrivilegedCommandRequest for PointerScanValidateRequest {
    type ResponseType = PointerScanValidateResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::PointerScan(PointerScanCommand::Validate {
            pointer_scan_validate_request: self.clone(),
        })
    }
}
