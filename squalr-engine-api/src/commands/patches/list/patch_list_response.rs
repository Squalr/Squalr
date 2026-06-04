use crate::commands::patches::patches_response::PatchesResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::patches::{PatchCommandStatus, PatchDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchListResponse {
    pub status: PatchCommandStatus,
    pub patches: Vec<PatchDescriptor>,
}

impl TypedPrivilegedCommandResponse for PatchListResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Patches(PatchesResponse::List {
            patch_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Patches(PatchesResponse::List { patch_list_response }) = response {
            Ok(patch_list_response)
        } else {
            Err(response)
        }
    }
}
