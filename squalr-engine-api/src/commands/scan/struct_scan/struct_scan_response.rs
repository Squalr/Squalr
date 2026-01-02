use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::structures::tasks::trackable_task_handle::TrackableTaskHandle;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StructScanResponse {
    pub trackable_task_handle: Option<TrackableTaskHandle>,
}

impl TypedPrivilegedCommandResponse for StructScanResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Scan(ScanResponse::StructScan {
            struct_scan_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Scan(ScanResponse::StructScan { struct_scan_response }) = response {
            Ok(struct_scan_response)
        } else {
            Err(response)
        }
    }
}
