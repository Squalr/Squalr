use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::plugins::list::{plugin_list_request::PluginListRequest, plugin_list_response::PluginListResponse};
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for PluginListRequest {
    type ResponseType = PluginListResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let opened_process_info = engine_privileged_state
            .get_process_manager()
            .get_opened_process();
        let plugins = engine_privileged_state
            .get_plugin_registry()
            .get_plugin_states(opened_process_info.as_ref());

        PluginListResponse { plugins, opened_process_info }
    }
}
