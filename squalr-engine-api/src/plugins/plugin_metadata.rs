use crate::plugins::PluginCapability;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginMetadata {
    plugin_id: String,
    display_name: String,
    description: String,
    plugin_capabilities: Vec<PluginCapability>,
    is_built_in: bool,
    is_enabled_by_default: bool,
}

impl PluginMetadata {
    pub fn new(
        plugin_id: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
        plugin_capabilities: Vec<PluginCapability>,
        is_built_in: bool,
        is_enabled_by_default: bool,
    ) -> Self {
        let mut plugin_capabilities = plugin_capabilities;
        plugin_capabilities.sort();
        plugin_capabilities.dedup();

        Self {
            plugin_id: plugin_id.into(),
            display_name: display_name.into(),
            description: description.into(),
            plugin_capabilities,
            is_built_in,
            is_enabled_by_default,
        }
    }

    pub fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_plugin_capabilities(&self) -> &[PluginCapability] {
        &self.plugin_capabilities
    }

    pub fn has_plugin_capability(
        &self,
        plugin_capability: PluginCapability,
    ) -> bool {
        self.plugin_capabilities.contains(&plugin_capability)
    }

    pub fn get_is_built_in(&self) -> bool {
        self.is_built_in
    }

    pub fn get_is_enabled_by_default(&self) -> bool {
        self.is_enabled_by_default
    }
}
