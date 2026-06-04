use crate::commands::patches::patches_command::PatchesCommand;
use crate::commands::patches::patches_response::PatchesResponse;
use crate::commands::patches::restore::patch_restore_response::PatchRestoreResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchRestoreRequest {
    pub patch_id: String,
}

impl PrivilegedCommandRequest for PatchRestoreRequest {
    type ResponseType = PatchRestoreResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Patches(PatchesCommand::Restore {
            patch_restore_request: self.clone(),
        })
    }
}

impl From<PatchRestoreResponse> for PatchesResponse {
    fn from(patch_restore_response: PatchRestoreResponse) -> Self {
        PatchesResponse::Restore { patch_restore_response }
    }
}
