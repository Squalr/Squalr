use crossbeam_channel::bounded;
use squalr_engine_api::commands::plugins::list::plugin_list_request::PluginListRequest;
use squalr_engine_api::commands::plugins::set_enabled::plugin_set_enabled_request::PluginSetEnabledRequest;
use squalr_engine_api::commands::plugins::set_order::plugin_set_order_request::PluginSetOrderRequest;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::plugins::{PluginConfiguration, PluginState};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

const PLUGIN_SYNC_TIMEOUT_SECONDS: u64 = 1;

pub struct PluginSnapshot {
    pub plugin_states: Vec<PluginState>,
    pub default_plugin_ids: Vec<String>,
}

pub fn get_plugin_snapshot(engine_execution_context: &Arc<dyn EngineExecutionContext>) -> Option<PluginSnapshot> {
    let plugin_list_request = PluginListRequest {};
    let (completion_sender, completion_receiver) = bounded(1);
    let did_send = match engine_execution_context.get_bindings().read() {
        Ok(engine_bindings) => plugin_list_request.send_unprivileged(&*engine_bindings, move |plugin_list_response| {
            let _ = completion_sender.send(PluginSnapshot {
                plugin_states: plugin_list_response.plugins,
                default_plugin_ids: plugin_list_response.default_plugin_ids,
            });
        }),
        Err(error) => {
            log::error!("Failed to acquire engine bindings while querying plugin state: {}", error);
            false
        }
    };

    if !did_send {
        return None;
    }

    match completion_receiver.recv_timeout(Duration::from_secs(PLUGIN_SYNC_TIMEOUT_SECONDS)) {
        Ok(plugin_snapshot) => Some(plugin_snapshot),
        Err(error) => {
            log::error!("Timed out waiting for plugin state list response: {}", error);
            None
        }
    }
}

pub fn apply_project_plugin_configuration(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    plugin_configuration: Option<&PluginConfiguration>,
) -> bool {
    let plugin_configuration = plugin_configuration.cloned().unwrap_or_default();
    let initial_plugin_snapshot = match get_plugin_snapshot(engine_execution_context) {
        Some(plugin_snapshot) => plugin_snapshot,
        None => {
            log::error!("Failed to query plugin state while applying project plugin overrides.");
            return false;
        }
    };
    let plugin_order = if plugin_configuration.get_priority_plugin_ids().is_empty() {
        initial_plugin_snapshot.default_plugin_ids
    } else {
        plugin_configuration.get_priority_plugin_ids().to_vec()
    };

    if !set_plugin_order(engine_execution_context, plugin_order) {
        return false;
    }

    let plugin_states = match get_plugin_snapshot(engine_execution_context) {
        Some(plugin_snapshot) => plugin_snapshot.plugin_states,
        None => {
            log::error!("Failed to query ordered plugin state while applying project plugin overrides.");
            return false;
        }
    };

    let enabled_plugin_ids = plugin_configuration
        .get_enabled_plugin_ids()
        .iter()
        .map(|plugin_id| plugin_id.as_str())
        .collect::<HashSet<_>>();
    let disabled_plugin_ids = plugin_configuration
        .get_disabled_plugin_ids()
        .iter()
        .map(|plugin_id| plugin_id.as_str())
        .collect::<HashSet<_>>();

    for plugin_state in plugin_states {
        let plugin_id = plugin_state.get_metadata().get_plugin_id();
        let should_enable = if enabled_plugin_ids.contains(plugin_id) {
            true
        } else if disabled_plugin_ids.contains(plugin_id) {
            false
        } else {
            plugin_state.get_metadata().get_is_enabled_by_default()
        };

        if plugin_state.get_is_enabled() == should_enable {
            continue;
        }

        if !set_plugin_enabled(engine_execution_context, plugin_id.to_string(), should_enable) {
            return false;
        }
    }

    true
}

fn set_plugin_order(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    plugin_ids: Vec<String>,
) -> bool {
    let plugin_set_order_request = PluginSetOrderRequest { plugin_ids };
    let (completion_sender, completion_receiver) = bounded(1);
    let did_send = match engine_execution_context.get_bindings().read() {
        Ok(engine_bindings) => plugin_set_order_request.send_unprivileged(&*engine_bindings, move |plugin_set_order_response| {
            let _ = completion_sender.send(plugin_set_order_response.did_update);
        }),
        Err(error) => {
            log::error!("Failed to acquire engine bindings while applying project plugin priority: {}", error);
            false
        }
    };

    if !did_send {
        return false;
    }

    match completion_receiver.recv_timeout(Duration::from_secs(PLUGIN_SYNC_TIMEOUT_SECONDS)) {
        Ok(_did_update) => true,
        Err(error) => {
            log::error!("Timed out waiting for plugin set-order response: {}", error);
            false
        }
    }
}

fn set_plugin_enabled(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    plugin_id: String,
    is_enabled: bool,
) -> bool {
    let plugin_set_enabled_request = PluginSetEnabledRequest { plugin_id, is_enabled };
    let (completion_sender, completion_receiver) = bounded(1);
    let did_send = match engine_execution_context.get_bindings().read() {
        Ok(engine_bindings) => plugin_set_enabled_request.send_unprivileged(&*engine_bindings, move |plugin_set_enabled_response| {
            let _ = completion_sender.send(plugin_set_enabled_response.did_update);
        }),
        Err(error) => {
            log::error!("Failed to acquire engine bindings while applying project plugin state: {}", error);
            false
        }
    };

    if !did_send {
        return false;
    }

    match completion_receiver.recv_timeout(Duration::from_secs(PLUGIN_SYNC_TIMEOUT_SECONDS)) {
        Ok(did_update) => did_update,
        Err(error) => {
            log::error!("Timed out waiting for plugin set-enabled response: {}", error);
            false
        }
    }
}
