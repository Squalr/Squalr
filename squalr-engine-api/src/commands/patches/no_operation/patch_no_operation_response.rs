use crate::commands::patches::patches_response::PatchesResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::patches::{PatchCommandStatus, PatchDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchNoOperationResponse {
    pub status: PatchCommandStatus,
    pub patch: Option<PatchDescriptor>,
}

impl TypedPrivilegedCommandResponse for PatchNoOperationResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Patches(PatchesResponse::NoOperation {
            patch_no_operation_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Patches(PatchesResponse::NoOperation { patch_no_operation_response }) = response {
            Ok(patch_no_operation_response)
        } else {
            Err(response)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PatchNoOperationResponse;
    use crate::commands::{
        patches::patches_response::PatchesResponse,
        privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse},
    };
    use crate::structures::patches::PatchCommandStatus;

    #[test]
    fn no_operation_patch_response_round_trips_through_privileged_response() {
        let patch_no_operation_response = PatchNoOperationResponse {
            status: PatchCommandStatus::success(),
            patch: None,
        };
        let engine_response = patch_no_operation_response.to_engine_response();

        assert!(matches!(
            engine_response,
            PrivilegedCommandResponse::Patches(PatchesResponse::NoOperation { .. })
        ));
        assert!(
            PatchNoOperationResponse::from_engine_response(engine_response)
                .expect("Expected no-operation patch response to round trip.")
                .status
                .get_success()
        );
    }
}
