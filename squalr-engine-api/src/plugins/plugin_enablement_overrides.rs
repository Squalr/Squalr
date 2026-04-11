use crate::plugins::PluginState;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct PluginEnablementOverrides {
    #[serde(default, rename = "enabled", skip_serializing_if = "Vec::is_empty")]
    enabled_plugin_ids: Vec<String>,
    #[serde(default, rename = "disabled", skip_serializing_if = "Vec::is_empty")]
    disabled_plugin_ids: Vec<String>,
}

impl PluginEnablementOverrides {
    pub fn new(
        enabled_plugin_ids: Vec<String>,
        disabled_plugin_ids: Vec<String>,
    ) -> Self {
        Self {
            enabled_plugin_ids: Self::normalize_plugin_ids(enabled_plugin_ids),
            disabled_plugin_ids: Self::normalize_plugin_ids(disabled_plugin_ids),
        }
    }

    pub fn from_plugin_states(plugin_states: &[PluginState]) -> Option<Self> {
        let mut enabled_plugin_ids = Vec::new();
        let mut disabled_plugin_ids = Vec::new();

        for plugin_state in plugin_states {
            let plugin_id = plugin_state.get_metadata().get_plugin_id().to_string();
            let is_enabled_by_default = plugin_state.get_metadata().get_is_enabled_by_default();

            match (is_enabled_by_default, plugin_state.get_is_enabled()) {
                (false, true) => enabled_plugin_ids.push(plugin_id),
                (true, false) => disabled_plugin_ids.push(plugin_id),
                _ => {}
            }
        }

        let plugin_enablement_overrides = Self::new(enabled_plugin_ids, disabled_plugin_ids);

        if plugin_enablement_overrides.is_empty() {
            None
        } else {
            Some(plugin_enablement_overrides)
        }
    }

    pub fn get_enabled_plugin_ids(&self) -> &[String] {
        &self.enabled_plugin_ids
    }

    pub fn get_disabled_plugin_ids(&self) -> &[String] {
        &self.disabled_plugin_ids
    }

    pub fn is_empty(&self) -> bool {
        self.enabled_plugin_ids.is_empty() && self.disabled_plugin_ids.is_empty()
    }

    fn normalize_plugin_ids(mut plugin_ids: Vec<String>) -> Vec<String> {
        plugin_ids.sort();
        plugin_ids.dedup();
        plugin_ids
    }
}

#[cfg(test)]
mod tests {
    use super::PluginEnablementOverrides;
    use crate::plugins::{PluginActivationState, PluginMetadata, PluginState};

    #[test]
    fn plugin_enablement_overrides_capture_only_non_default_states() {
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

        let plugin_enablement_overrides =
            PluginEnablementOverrides::from_plugin_states(&plugin_states).expect("Expected non-default plugin states to produce overrides.");

        assert_eq!(plugin_enablement_overrides.get_enabled_plugin_ids(), &[String::from("builtin.default-off")]);
        assert_eq!(plugin_enablement_overrides.get_disabled_plugin_ids(), &[String::from("builtin.disabled")]);
    }
}
