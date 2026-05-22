use crate::commands::pointer_scan::expand::pointer_scan_expand_response::PointerScanExpandResponse;
use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointerScanExpandRequest {
    pub session_id: u64,
    pub parent_node_id: Option<u64>,
    pub page_index: u64,
    pub page_size: u64,
}

impl PrivilegedCommandRequest for PointerScanExpandRequest {
    type ResponseType = PointerScanExpandResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::PointerScan(PointerScanCommand::Expand {
            pointer_scan_expand_request: self.clone(),
        })
    }
}
