use crate::commands::patches::patches_response::PatchesResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::patches::{PatchCommandStatus, PatchDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchRestoreResponse {
    pub status: PatchCommandStatus,
    pub patch: Option<PatchDescriptor>,
}

impl TypedPrivilegedCommandResponse for PatchRestoreResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Patches(PatchesResponse::Restore {
            patch_restore_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Patches(PatchesResponse::Restore { patch_restore_response }) = response {
            Ok(patch_restore_response)
        } else {
            Err(response)
        }
    }
}
