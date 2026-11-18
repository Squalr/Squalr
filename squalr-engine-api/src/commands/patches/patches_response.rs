use crate::commands::patches::{
    apply::patch_apply_response::PatchApplyResponse, list::patch_list_response::PatchListResponse,
    no_operation::patch_no_operation_response::PatchNoOperationResponse, restore::patch_restore_response::PatchRestoreResponse,
    restore_address::patch_restore_address_response::PatchRestoreAddressResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PatchesResponse {
    Apply {
        patch_apply_response: PatchApplyResponse,
    },
    NoOperation {
        patch_no_operation_response: PatchNoOperationResponse,
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

impl PatchesResponse {
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::Apply { patch_apply_response: reply } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::NoOperation {
                patch_no_operation_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::Restore { patch_restore_response: reply } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::RestoreAddress {
                patch_restore_address_response: reply,
            } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            Self::List { patch_list_response: reply } if !reply.status.get_success() => Some(reply.status.get_message().unwrap_or("The command failed.")),
            _ => None,
        }
    }
}
