use crossbeam_channel::bounded;
use squalr_engine_api::commands::plugins::list::plugin_list_request::PluginListRequest;
use squalr_engine_api::commands::plugins::set_enabled::plugin_set_enabled_request::PluginSetEnabledRequest;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::plugins::{PluginEnablementOverrides, PluginState};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

const PLUGIN_SYNC_TIMEOUT_SECONDS: u64 = 1;

pub fn get_plugin_states(engine_execution_context: &Arc<dyn EngineExecutionContext>) -> Option<Vec<PluginState>> {
    let plugin_list_request = PluginListRequest {};
    let (completion_sender, completion_receiver) = bounded(1);
    let did_send = match engine_execution_context.get_bindings().read() {
        Ok(engine_bindings) => plugin_list_request.send_unprivileged(&*engine_bindings, move |plugin_list_response| {
            let _ = completion_sender.send(plugin_list_response.plugins);
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
        Ok(plugin_states) => Some(plugin_states),
        Err(error) => {
            log::error!("Timed out waiting for plugin state list response: {}", error);
            None
        }
    }
}

pub fn apply_project_plugin_configuration(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    plugin_enablement_overrides: Option<&PluginEnablementOverrides>,
) -> bool {
    let Some(plugin_enablement_overrides) = plugin_enablement_overrides else {
        return true;
    };

    let plugin_states = match get_plugin_states(engine_execution_context) {
        Some(plugin_states) => plugin_states,
        None => {
            log::error!("Failed to query plugin state while applying project plugin overrides.");
            return false;
        }
    };

    let enabled_plugin_ids = plugin_enablement_overrides
        .get_enabled_plugin_ids()
        .iter()
        .map(|plugin_id| plugin_id.as_str())
        .collect::<HashSet<_>>();
    let disabled_plugin_ids = plugin_enablement_overrides
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
