use crate::commands::patches::list::patch_list_response::PatchListResponse;
use crate::commands::patches::patches_command::PatchesCommand;
use crate::commands::patches::patches_response::PatchesResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PatchListRequest;

impl PrivilegedCommandRequest for PatchListRequest {
    type ResponseType = PatchListResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Patches(PatchesCommand::List {
            patch_list_request: self.clone(),
        })
    }
}

impl From<PatchListResponse> for PatchesResponse {
    fn from(patch_list_response: PatchListResponse) -> Self {
        PatchesResponse::List { patch_list_response }
    }
}
