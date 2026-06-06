use crate::commands::patches::patches_response::PatchesResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::patches::{PatchCommandStatus, PatchDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchRestoreAddressResponse {
    pub status: PatchCommandStatus,
    pub patch: Option<PatchDescriptor>,
}

impl TypedPrivilegedCommandResponse for PatchRestoreAddressResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Patches(PatchesResponse::RestoreAddress {
            patch_restore_address_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Patches(PatchesResponse::RestoreAddress {
            patch_restore_address_response,
        }) = response
        {
            Ok(patch_restore_address_response)
        } else {
            Err(response)
        }
    }
}
