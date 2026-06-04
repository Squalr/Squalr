use crate::commands::patches::{
    apply::patch_apply_request::PatchApplyRequest, list::patch_list_request::PatchListRequest, restore::patch_restore_request::PatchRestoreRequest,
    restore_address::patch_restore_address_request::PatchRestoreAddressRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PatchesCommand {
    Apply {
        patch_apply_request: PatchApplyRequest,
    },
    Restore {
        patch_restore_request: PatchRestoreRequest,
    },
    RestoreAddress {
        patch_restore_address_request: PatchRestoreAddressRequest,
    },
    List {
        patch_list_request: PatchListRequest,
    },
}
