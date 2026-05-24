use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::pointer_scan::start::pointer_scan_start_response::PointerScanStartResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use crate::structures::pointer_scans::pointer_scan_target_request::PointerScanTargetRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointerScanStartRequest {
    pub target: PointerScanTargetRequest,
    pub pointer_size: PointerScanPointerSize,
    pub max_depth: u64,
    pub offset_radius: u64,
    pub address_space: PointerScanAddressSpace,
}

impl PrivilegedCommandRequest for PointerScanStartRequest {
    type ResponseType = PointerScanStartResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::PointerScan(PointerScanCommand::Start {
            pointer_scan_start_request: self.clone(),
        })
    }
}
