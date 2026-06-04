use crate::commands::patches::patches_command::PatchesCommand;
use crate::commands::patches::patches_response::PatchesResponse;
use crate::commands::patches::restore_address::patch_restore_address_response::PatchRestoreAddressResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchRestoreAddressRequest {
    pub address: u64,
    pub module_name: String,
}

impl PrivilegedCommandRequest for PatchRestoreAddressRequest {
    type ResponseType = PatchRestoreAddressResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Patches(PatchesCommand::RestoreAddress {
            patch_restore_address_request: self.clone(),
        })
    }
}

impl From<PatchRestoreAddressResponse> for PatchesResponse {
    fn from(patch_restore_address_response: PatchRestoreAddressResponse) -> Self {
        PatchesResponse::RestoreAddress {
            patch_restore_address_response,
        }
    }
}
