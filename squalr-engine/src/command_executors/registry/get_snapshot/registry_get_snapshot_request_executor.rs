use crate::{command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor, engine_privileged_state::EnginePrivilegedState};
use squalr_engine_api::commands::registry::get_snapshot::{
    registry_get_snapshot_request::RegistryGetSnapshotRequest, registry_get_snapshot_response::RegistryGetSnapshotResponse,
};
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for RegistryGetSnapshotRequest {
    type ResponseType = RegistryGetSnapshotResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        RegistryGetSnapshotResponse {
            symbol_registry_snapshot: engine_privileged_state.get_symbol_registry_snapshot(),
        }
    }
}
