use crate::views::plugins::pane_state::PluginsPaneState;
use squalr_engine_api::plugins::PluginKind;

pub fn build_plugins_summary_lines_with_capacity(
    plugins_pane_state: &PluginsPaneState,
    line_capacity: usize,
    opened_process_name: Option<&str>,
    opened_process_identifier: Option<u32>,
) -> Vec<String> {
    if line_capacity == 0 {
        return Vec::new();
    }

    let enabled_plugin_count = plugins_pane_state
        .plugins
        .iter()
        .filter(|plugin_state| plugin_state.get_is_enabled())
        .count();
    let eligible_plugin_count = plugins_pane_state
        .plugins
        .iter()
        .filter(|plugin_state| plugin_state.get_can_activate_for_current_process())
        .count();
    let active_plugin_count = plugins_pane_state
        .plugins
        .iter()
        .filter(|plugin_state| plugin_state.get_is_active_for_current_process())
        .count();

    let mut summary_lines = vec![
        "[NAV] Up/Down move | Home/End jump.".to_string(),
        "[ACT] Space/Enter toggle | r refresh.".to_string(),
        format!("[PROC] {}.", format_process_context(opened_process_name, opened_process_identifier)),
        format!(
            "[PLUG] total={} | enabled={} | eligible={} | active={}.",
            plugins_pane_state.plugins.len(),
            enabled_plugin_count,
            eligible_plugin_count,
            active_plugin_count
        ),
    ];

    if let Some(selected_plugin) = plugins_pane_state.selected_plugin() {
        let metadata = selected_plugin.get_metadata();
        summary_lines.push(format!(
            "[SEL] {} | kind={} | built_in={}.",
            metadata.get_display_name(),
            plugin_kind_label(metadata.get_plugin_kind()),
            metadata.get_is_built_in()
        ));
        summary_lines.push(format!(
            "[ROUTE] enabled={} | eligible={} | active={}.",
            selected_plugin.get_is_enabled(),
            selected_plugin.get_can_activate_for_current_process(),
            selected_plugin.get_is_active_for_current_process()
        ));
        summary_lines.push(format!("[ID] {}.", metadata.get_plugin_id()));
        summary_lines.push(format!("[DESC] {}.", metadata.get_description()));
    } else {
        summary_lines.push("[SEL] none.".to_string());
    }

    summary_lines.push(format!("[STAT] {}.", plugins_pane_state.status_message));

    summary_lines.into_iter().take(line_capacity).collect()
}

fn format_process_context(
    opened_process_name: Option<&str>,
    opened_process_identifier: Option<u32>,
) -> String {
    match (opened_process_name, opened_process_identifier) {
        (Some(opened_process_name), Some(opened_process_identifier)) => {
            format!("target={} | pid={}", opened_process_name, opened_process_identifier)
        }
        (Some(opened_process_name), None) => format!("target={}", opened_process_name),
        (None, Some(opened_process_identifier)) => format!("pid={}", opened_process_identifier),
        (None, None) => "target=none".to_string(),
    }
}

fn plugin_kind_label(plugin_kind: PluginKind) -> &'static str {
    match plugin_kind {
        PluginKind::MemoryView => "memory-view",
    }
}
