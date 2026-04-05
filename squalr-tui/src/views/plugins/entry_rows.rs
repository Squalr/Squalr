use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::plugins::pane_state::PluginsPaneState;
use squalr_engine_api::plugins::{PluginActivationState, PluginKind, PluginState};

pub fn build_visible_plugin_entry_rows(
    plugins_pane_state: &PluginsPaneState,
    viewport_capacity: usize,
) -> Vec<PaneEntryRow> {
    if viewport_capacity == 0 {
        return Vec::new();
    }

    let visible_plugin_range = build_selection_relative_viewport_range(
        plugins_pane_state.plugins.len(),
        plugins_pane_state.selected_plugin_list_index,
        viewport_capacity,
    );
    let mut entry_rows = Vec::with_capacity(visible_plugin_range.len());

    for visible_plugin_index in visible_plugin_range {
        if let Some(plugin_state) = plugins_pane_state.plugins.get(visible_plugin_index) {
            let metadata = plugin_state.get_metadata();
            let is_selected_plugin = plugins_pane_state.selected_plugin_list_index == Some(visible_plugin_index);
            let marker_text = build_plugin_marker_text(is_selected_plugin, plugin_state.get_activation_state(), plugin_state.get_is_enabled());
            let secondary_text = Some(format!(
                "{} | {} | {} | {}",
                plugin_kind_label(metadata.get_plugin_kind()),
                if metadata.get_is_built_in() { "built-in" } else { "external" },
                plugin_status_label(plugin_state),
                metadata.get_description()
            ));

            let entry_row = if is_selected_plugin {
                PaneEntryRow::selected(marker_text, metadata.get_display_name().to_string(), secondary_text)
            } else if !plugin_state.get_is_enabled() {
                PaneEntryRow::disabled(marker_text, metadata.get_display_name().to_string(), secondary_text)
            } else {
                PaneEntryRow::normal(marker_text, metadata.get_display_name().to_string(), secondary_text)
            };

            entry_rows.push(entry_row);
        }
    }

    entry_rows
}

fn build_plugin_marker_text(
    is_selected_plugin: bool,
    activation_state: PluginActivationState,
    is_enabled: bool,
) -> String {
    let mut marker_text = String::new();
    if is_selected_plugin {
        marker_text.push('>');
    }
    if matches!(activation_state, PluginActivationState::Activated) {
        marker_text.push('*');
    } else if matches!(activation_state, PluginActivationState::Activating) {
        marker_text.push('~');
    }
    if !is_enabled {
        marker_text.push('x');
    }

    marker_text
}

fn plugin_status_label(plugin_state: &PluginState) -> &'static str {
    if !plugin_state.get_is_enabled() {
        return "disabled";
    }

    match plugin_state.get_activation_state() {
        PluginActivationState::Idle => "idle",
        PluginActivationState::Available => "available",
        PluginActivationState::Activating => "activating",
        PluginActivationState::Activated => "activated",
    }
}

fn plugin_kind_label(plugin_kind: PluginKind) -> &'static str {
    match plugin_kind {
        PluginKind::MemoryView => "memory-view",
    }
}

#[cfg(test)]
mod tests {
    use super::build_visible_plugin_entry_rows;
    use crate::state::pane_entry_row::PaneEntryRowTone;
    use crate::views::plugins::pane_state::PluginsPaneState;
    use squalr_engine_api::plugins::{PluginActivationState, PluginKind, PluginMetadata, PluginState};

    fn create_plugin_state(
        plugin_id: &str,
        display_name: &str,
        is_enabled: bool,
        activation_state: PluginActivationState,
    ) -> PluginState {
        PluginState::new(
            PluginMetadata::new(plugin_id, display_name, "Test plugin", PluginKind::MemoryView, true),
            is_enabled,
            activation_state,
        )
    }

    #[test]
    fn disabled_plugins_render_selected_marker_with_disabled_suffix() {
        let mut plugins_pane_state = PluginsPaneState::default();
        plugins_pane_state.apply_plugin_states(vec![create_plugin_state(
            "disabled",
            "Disabled",
            false,
            PluginActivationState::Idle,
        )]);

        let visible_entry_rows = build_visible_plugin_entry_rows(&plugins_pane_state, 4);

        assert_eq!(visible_entry_rows.len(), 1);
        assert_eq!(visible_entry_rows[0].tone, PaneEntryRowTone::Selected);
        assert_eq!(visible_entry_rows[0].marker_text, ">x");
    }
}
