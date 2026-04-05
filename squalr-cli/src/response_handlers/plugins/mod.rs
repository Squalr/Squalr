use squalr_engine_api::commands::plugins::list::plugin_list_response::PluginListResponse;
use squalr_engine_api::commands::plugins::plugins_response::PluginsResponse;
use squalr_engine_api::commands::plugins::set_enabled::plugin_set_enabled_response::PluginSetEnabledResponse;
use squalr_engine_api::plugins::{PluginKind, PluginState};
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;

pub fn handle_plugins_response(response: PluginsResponse) {
    match response {
        PluginsResponse::List { plugin_list_response } => handle_plugin_list_response(plugin_list_response),
        PluginsResponse::SetEnabled { plugin_set_enabled_response } => handle_plugin_set_enabled_response(plugin_set_enabled_response),
    }
}

fn handle_plugin_list_response(plugin_list_response: PluginListResponse) {
    log_plugin_inventory("Plugins", &plugin_list_response.plugins, plugin_list_response.opened_process_info.as_ref());
}

fn handle_plugin_set_enabled_response(plugin_set_enabled_response: PluginSetEnabledResponse) {
    if plugin_set_enabled_response.did_update {
        log::info!("Plugin enablement updated.");
    } else {
        log::info!("Plugin enablement was unchanged.");
    }

    log_plugin_inventory(
        "Plugins",
        &plugin_set_enabled_response.plugins,
        plugin_set_enabled_response.opened_process_info.as_ref(),
    );
}

fn log_plugin_inventory(
    inventory_label: &str,
    plugins: &[PluginState],
    opened_process_info: Option<&OpenedProcessInfo>,
) {
    log::info!(
        "{}: count={} | target_process={}",
        inventory_label,
        plugins.len(),
        format_opened_process_context(opened_process_info)
    );

    if plugins.is_empty() {
        log::info!("No plugins are registered.");
        return;
    }

    for plugin_state in plugins {
        let metadata = plugin_state.get_metadata();
        log::info!(
            "{} | id={} | kind={} | built_in={} | enabled={} | eligible={} | active={}",
            metadata.get_display_name(),
            metadata.get_plugin_id(),
            plugin_kind_label(metadata.get_plugin_kind()),
            metadata.get_is_built_in(),
            plugin_state.get_is_enabled(),
            plugin_state.get_can_activate_for_current_process(),
            plugin_state.get_is_active_for_current_process()
        );
        log::info!("  {}", metadata.get_description());
    }
}

fn format_opened_process_context(opened_process_info: Option<&OpenedProcessInfo>) -> String {
    match opened_process_info {
        Some(opened_process_info) => {
            format!("{} (pid={})", opened_process_info.get_name(), opened_process_info.get_process_id_raw())
        }
        None => "none".to_string(),
    }
}

fn plugin_kind_label(plugin_kind: PluginKind) -> &'static str {
    match plugin_kind {
        PluginKind::MemoryView => "memory-view",
    }
}
