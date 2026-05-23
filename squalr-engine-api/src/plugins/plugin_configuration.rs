use crate::plugins::PluginState;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct PluginConfiguration {
    #[serde(default, rename = "enabled", skip_serializing_if = "Vec::is_empty")]
    enabled_plugin_ids: Vec<String>,
    #[serde(default, rename = "disabled", skip_serializing_if = "Vec::is_empty")]
    disabled_plugin_ids: Vec<String>,
    #[serde(default, rename = "priority", skip_serializing_if = "Vec::is_empty")]
    priority_plugin_ids: Vec<String>,
}

impl PluginConfiguration {
    pub fn new(
        enabled_plugin_ids: Vec<String>,
        disabled_plugin_ids: Vec<String>,
        priority_plugin_ids: Vec<String>,
    ) -> Self {
        Self {
            enabled_plugin_ids: Self::normalize_sorted_plugin_ids(enabled_plugin_ids),
            disabled_plugin_ids: Self::normalize_sorted_plugin_ids(disabled_plugin_ids),
            priority_plugin_ids: Self::normalize_ordered_plugin_ids(priority_plugin_ids),
        }
    }

    pub fn from_plugin_states(
        plugin_states: &[PluginState],
        default_plugin_ids: &[String],
    ) -> Option<Self> {
        let mut enabled_plugin_ids = Vec::new();
        let mut disabled_plugin_ids = Vec::new();
        let current_plugin_ids = plugin_states
            .iter()
            .map(|plugin_state| plugin_state.get_metadata().get_plugin_id().to_string())
            .collect::<Vec<_>>();

        for plugin_state in plugin_states {
            let plugin_id = plugin_state.get_metadata().get_plugin_id().to_string();
            let is_enabled_by_default = plugin_state.get_metadata().get_is_enabled_by_default();

            match (is_enabled_by_default, plugin_state.get_is_enabled()) {
                (false, true) => enabled_plugin_ids.push(plugin_id),
                (true, false) => disabled_plugin_ids.push(plugin_id),
                _ => {}
            }
        }

        let priority_plugin_ids = if default_plugin_ids.is_empty() || current_plugin_ids == default_plugin_ids {
            Vec::new()
        } else {
            current_plugin_ids
        };
        let plugin_configuration = Self::new(enabled_plugin_ids, disabled_plugin_ids, priority_plugin_ids);

        if plugin_configuration.is_empty() { None } else { Some(plugin_configuration) }
    }

    pub fn get_enabled_plugin_ids(&self) -> &[String] {
        &self.enabled_plugin_ids
    }

    pub fn get_disabled_plugin_ids(&self) -> &[String] {
        &self.disabled_plugin_ids
    }

    pub fn get_priority_plugin_ids(&self) -> &[String] {
        &self.priority_plugin_ids
    }

    pub fn is_empty(&self) -> bool {
        self.enabled_plugin_ids.is_empty() && self.disabled_plugin_ids.is_empty() && self.priority_plugin_ids.is_empty()
    }

    fn normalize_sorted_plugin_ids(mut plugin_ids: Vec<String>) -> Vec<String> {
        plugin_ids.retain(|plugin_id| !plugin_id.is_empty());
        plugin_ids.sort();
        plugin_ids.dedup();
        plugin_ids
    }

    fn normalize_ordered_plugin_ids(plugin_ids: Vec<String>) -> Vec<String> {
        let mut seen_plugin_ids = HashSet::new();
        let mut normalized_plugin_ids = Vec::new();

        for plugin_id in plugin_ids {
            if plugin_id.is_empty() || !seen_plugin_ids.insert(plugin_id.clone()) {
                continue;
            }

            normalized_plugin_ids.push(plugin_id);
        }

        normalized_plugin_ids
    }
}

#[cfg(test)]
mod tests {
    use super::PluginConfiguration;
    use crate::plugins::{PluginActivationState, PluginMetadata, PluginState};

    #[test]
    fn plugin_configuration_captures_only_non_default_enablement() {
        let plugin_states = vec![
            PluginState::new(
                PluginMetadata::new("builtin.default-on", "Default On", "", vec![], true, true),
                true,
                PluginActivationState::Idle,
            ),
            PluginState::new(
                PluginMetadata::new("builtin.default-off", "Default Off", "", vec![], true, false),
                true,
                PluginActivationState::Idle,
            ),
            PluginState::new(
                PluginMetadata::new("builtin.disabled", "Disabled", "", vec![], true, true),
                false,
                PluginActivationState::Idle,
            ),
        ];
        let default_plugin_ids = plugin_states
            .iter()
            .map(|plugin_state| plugin_state.get_metadata().get_plugin_id().to_string())
            .collect::<Vec<_>>();

        let plugin_configuration =
            PluginConfiguration::from_plugin_states(&plugin_states, &default_plugin_ids).expect("Expected non-default plugin states to produce configuration.");

        assert_eq!(plugin_configuration.get_enabled_plugin_ids(), &[String::from("builtin.default-off")]);
        assert_eq!(plugin_configuration.get_disabled_plugin_ids(), &[String::from("builtin.disabled")]);
        assert!(plugin_configuration.get_priority_plugin_ids().is_empty());
    }

    #[test]
    fn plugin_configuration_captures_priority_when_order_differs_from_default() {
        let plugin_states = vec![
            PluginState::new(
                PluginMetadata::new("builtin.second", "Second", "", vec![], true, true),
                true,
                PluginActivationState::Idle,
            ),
            PluginState::new(
                PluginMetadata::new("builtin.first", "First", "", vec![], true, true),
                true,
                PluginActivationState::Idle,
            ),
        ];
        let default_plugin_ids = vec![String::from("builtin.first"), String::from("builtin.second")];

        let plugin_configuration = PluginConfiguration::from_plugin_states(&plugin_states, &default_plugin_ids)
            .expect("Expected non-default plugin priority to produce configuration.");

        assert_eq!(
            plugin_configuration.get_priority_plugin_ids(),
            &[String::from("builtin.second"), String::from("builtin.first")]
        );
    }
}
