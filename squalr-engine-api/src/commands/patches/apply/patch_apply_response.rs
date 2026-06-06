use crate::commands::patches::patches_response::PatchesResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::patches::{PatchCommandStatus, PatchDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchApplyResponse {
    pub status: PatchCommandStatus,
    pub patch: Option<PatchDescriptor>,
}

impl TypedPrivilegedCommandResponse for PatchApplyResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Patches(PatchesResponse::Apply {
            patch_apply_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Patches(PatchesResponse::Apply { patch_apply_response }) = response {
            Ok(patch_apply_response)
        } else {
            Err(response)
        }
    }
}
