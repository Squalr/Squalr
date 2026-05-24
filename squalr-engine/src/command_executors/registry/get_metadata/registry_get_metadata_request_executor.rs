use crate::{command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor, engine_privileged_state::EnginePrivilegedState};
use squalr_engine_api::commands::registry::get_metadata::{
    registry_get_metadata_request::RegistryGetMetadataRequest, registry_get_metadata_response::RegistryGetMetadataResponse,
};
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for RegistryGetMetadataRequest {
    type ResponseType = RegistryGetMetadataResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        RegistryGetMetadataResponse {
            privileged_registry_catalog: engine_privileged_state.get_privileged_registry_catalog(),
        }
    }
}
