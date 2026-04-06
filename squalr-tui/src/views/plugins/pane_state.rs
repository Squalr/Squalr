use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::plugins::entry_rows::build_visible_plugin_entry_rows;
use crate::views::plugins::summary::build_plugins_summary_lines_with_capacity;
use squalr_engine_api::plugins::PluginState;

#[derive(Clone, Debug)]
pub struct PluginsPaneState {
    pub plugins: Vec<PluginState>,
    pub selected_plugin_list_index: Option<usize>,
    pub has_loaded_plugins_once: bool,
    pub is_refreshing_plugins: bool,
    pub is_updating_plugin_enabled: bool,
    pub status_message: String,
}

impl PluginsPaneState {
    pub fn apply_plugin_states(
        &mut self,
        plugins: Vec<PluginState>,
    ) {
        let selected_plugin_id_before_refresh = self.selected_plugin_id();
        self.plugins = plugins;
        self.selected_plugin_list_index = selected_plugin_id_before_refresh
            .as_deref()
            .and_then(|selected_plugin_id| {
                self.plugins
                    .iter()
                    .position(|plugin_state| plugin_state.get_metadata().get_plugin_id() == selected_plugin_id)
            })
            .or_else(|| if self.plugins.is_empty() { None } else { Some(0) });
        self.has_loaded_plugins_once = true;
    }

    pub fn selected_plugin(&self) -> Option<&PluginState> {
        self.selected_plugin_list_index
            .and_then(|selected_plugin_list_index| self.plugins.get(selected_plugin_list_index))
    }

    pub fn selected_plugin_id(&self) -> Option<String> {
        self.selected_plugin()
            .map(|plugin_state| plugin_state.get_metadata().get_plugin_id().to_string())
    }

    pub fn select_next_plugin(&mut self) {
        if self.plugins.is_empty() {
            self.selected_plugin_list_index = None;
            return;
        }

        let selected_plugin_list_index = self.selected_plugin_list_index.unwrap_or(0);
        self.selected_plugin_list_index = Some((selected_plugin_list_index + 1) % self.plugins.len());
    }

    pub fn select_previous_plugin(&mut self) {
        if self.plugins.is_empty() {
            self.selected_plugin_list_index = None;
            return;
        }

        let selected_plugin_list_index = self.selected_plugin_list_index.unwrap_or(0);
        self.selected_plugin_list_index = Some(if selected_plugin_list_index == 0 {
            self.plugins.len() - 1
        } else {
            selected_plugin_list_index - 1
        });
    }

    pub fn select_first_plugin(&mut self) {
        self.selected_plugin_list_index = if self.plugins.is_empty() { None } else { Some(0) };
    }

    pub fn select_last_plugin(&mut self) {
        self.selected_plugin_list_index = self.plugins.len().checked_sub(1);
    }

    pub fn summary_lines_with_capacity(
        &self,
        line_capacity: usize,
    ) -> Vec<String> {
        build_plugins_summary_lines_with_capacity(self, line_capacity)
    }

    pub fn visible_plugin_entry_rows(
        &self,
        viewport_capacity: usize,
    ) -> Vec<PaneEntryRow> {
        build_visible_plugin_entry_rows(self, viewport_capacity)
    }
}

impl Default for PluginsPaneState {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            selected_plugin_list_index: None,
            has_loaded_plugins_once: false,
            is_refreshing_plugins: false,
            is_updating_plugin_enabled: false,
            status_message: "Ready.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PluginsPaneState;
    use squalr_engine_api::plugins::{PluginActivationState, PluginKind, PluginMetadata, PluginState};

    fn create_plugin_state(plugin_id: &str) -> PluginState {
        PluginState::new(
            PluginMetadata::new(plugin_id, plugin_id, "Plugin", PluginKind::MemoryView, true, true),
            true,
            PluginActivationState::Idle,
        )
    }

    #[test]
    fn apply_plugin_states_restores_selection_by_plugin_id() {
        let mut plugins_pane_state = PluginsPaneState::default();
        plugins_pane_state.apply_plugin_states(vec![create_plugin_state("plugin-a"), create_plugin_state("plugin-b")]);
        plugins_pane_state.select_next_plugin();

        plugins_pane_state.apply_plugin_states(vec![create_plugin_state("plugin-c"), create_plugin_state("plugin-b")]);

        assert_eq!(plugins_pane_state.selected_plugin_id().as_deref(), Some("plugin-b"));
    }
}
