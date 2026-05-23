use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::plugins::set_order::{plugin_set_order_request::PluginSetOrderRequest, plugin_set_order_response::PluginSetOrderResponse};
use squalr_engine_api::events::plugins::changed::plugins_changed_event::PluginsChangedEvent;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for PluginSetOrderRequest {
    type ResponseType = PluginSetOrderResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let plugin_registry = engine_privileged_state.get_plugin_registry();
        let did_update = plugin_registry.set_plugin_order(self.plugin_ids.clone());

        if did_update {
            engine_privileged_state.invalidate_memory_view_runtime_state();
            engine_privileged_state.notify_registry_changed();
            engine_privileged_state.emit_event(PluginsChangedEvent {});
        }

        let opened_process_info = engine_privileged_state
            .get_process_manager()
            .get_opened_process();
        let plugins = engine_privileged_state.get_plugin_states();
        let default_plugin_ids = plugin_registry.get_default_plugin_ids();

        PluginSetOrderResponse {
            plugins,
            opened_process_info,
            default_plugin_ids,
            did_update,
        }
    }
}
