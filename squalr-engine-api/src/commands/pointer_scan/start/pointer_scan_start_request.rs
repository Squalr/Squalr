use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::pointer_scan::start::pointer_scan_start_response::PointerScanStartResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct PointerScanStartRequest {
    #[structopt(short = "a", long)]
    pub target_address: AnonymousValueString,
    #[structopt(short = "s", long)]
    pub pointer_size: PointerScanPointerSize,
    #[structopt(short = "d", long)]
    pub max_depth: u64,
    #[structopt(short = "o", long)]
    pub offset_radius: u64,
}

impl PrivilegedCommandRequest for PointerScanStartRequest {
    type ResponseType = PointerScanStartResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::PointerScan(PointerScanCommand::Start {
            pointer_scan_start_request: self.clone(),
        })
    }
}
