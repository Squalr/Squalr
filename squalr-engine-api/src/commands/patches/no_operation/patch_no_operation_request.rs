use crate::commands::patches::no_operation::patch_no_operation_response::PatchNoOperationResponse;
use crate::commands::patches::patches_command::PatchesCommand;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchNoOperationRequest {
    pub address: u64,
    pub module_name: String,
    pub instruction_bytes_hint: Option<Vec<u8>>,
    pub label: Option<String>,
}

impl PrivilegedCommandRequest for PatchNoOperationRequest {
    type ResponseType = PatchNoOperationResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Patches(PatchesCommand::NoOperation {
            patch_no_operation_request: self.clone(),
        })
    }
}
