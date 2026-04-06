use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::plugins::set_enabled::{
    plugin_set_enabled_request::PluginSetEnabledRequest, plugin_set_enabled_response::PluginSetEnabledResponse,
};
use squalr_engine_api::plugins::PluginCapability;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for PluginSetEnabledRequest {
    type ResponseType = PluginSetEnabledResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let plugin_registry = engine_privileged_state.get_plugin_registry();
        let did_update = plugin_registry.set_plugin_enabled(&self.plugin_id, self.is_enabled);
        if did_update {
            if plugin_registry.has_plugin_capability(&self.plugin_id, PluginCapability::MemoryView) {
                engine_privileged_state.invalidate_memory_view_runtime_state();
            }

            if plugin_registry.has_plugin_capability(&self.plugin_id, PluginCapability::DataType) {
                engine_privileged_state.notify_registry_changed();
            }
        }
        let opened_process_info = engine_privileged_state
            .get_process_manager()
            .get_opened_process();
        let plugins = engine_privileged_state.get_plugin_states();

        PluginSetEnabledResponse {
            plugins,
            opened_process_info,
            did_update,
        }
    }
}
