use crate::{command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor, engine_privileged_state::EnginePrivilegedState};
use squalr_engine_api::commands::registry::set_project_symbols::{
    registry_set_project_symbols_request::RegistrySetProjectSymbolsRequest, registry_set_project_symbols_response::RegistrySetProjectSymbolsResponse,
};
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for RegistrySetProjectSymbolsRequest {
    type ResponseType = RegistrySetProjectSymbolsResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        RegistrySetProjectSymbolsResponse {
            success: engine_privileged_state.set_project_symbol_catalog(&self.project_symbol_catalog),
        }
    }
}
