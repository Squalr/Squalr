use crate::commands::patches::apply::patch_apply_response::PatchApplyResponse;
use crate::commands::patches::patches_command::PatchesCommand;
use crate::commands::patches::patches_response::PatchesResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::structures::patches::PatchKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchApplyRequest {
    pub address: u64,
    pub module_name: String,
    pub patched_bytes: Vec<u8>,
    pub kind: PatchKind,
    pub label: Option<String>,
}

impl PrivilegedCommandRequest for PatchApplyRequest {
    type ResponseType = PatchApplyResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Patches(PatchesCommand::Apply {
            patch_apply_request: self.clone(),
        })
    }
}

impl From<PatchApplyResponse> for PatchesResponse {
    fn from(patch_apply_response: PatchApplyResponse) -> Self {
        PatchesResponse::Apply { patch_apply_response }
    }
}
