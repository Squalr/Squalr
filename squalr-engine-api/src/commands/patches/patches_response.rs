use crate::commands::patches::{
    apply::patch_apply_response::PatchApplyResponse, list::patch_list_response::PatchListResponse, restore::patch_restore_response::PatchRestoreResponse,
    restore_address::patch_restore_address_response::PatchRestoreAddressResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PatchesResponse {
    Apply {
        patch_apply_response: PatchApplyResponse,
    },
    Restore {
        patch_restore_response: PatchRestoreResponse,
    },
    RestoreAddress {
        patch_restore_address_response: PatchRestoreAddressResponse,
    },
    List {
        patch_list_response: PatchListResponse,
    },
}
