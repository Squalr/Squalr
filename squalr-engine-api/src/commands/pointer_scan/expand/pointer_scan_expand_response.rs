use crate::commands::pointer_scan::pointer_scan_response::PointerScanResponse;
use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PointerScanExpandResponse {
    pub session_id: u64,
    pub parent_node_id: Option<u64>,
    pub page_index: u64,
    pub last_page_index: u64,
    pub total_node_count: u64,
    pub pointer_scan_nodes: Vec<PointerScanNode>,
}

impl TypedPrivilegedCommandResponse for PointerScanExpandResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::PointerScan(PointerScanResponse::Expand {
            pointer_scan_expand_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::PointerScan(PointerScanResponse::Expand { pointer_scan_expand_response }) = response {
            Ok(pointer_scan_expand_response)
        } else {
            Err(response)
        }
    }
}
