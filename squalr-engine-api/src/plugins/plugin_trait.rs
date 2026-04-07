use crate::plugins::{PluginCapability, PluginMetadata};

pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;

    fn supports_capability(
        &self,
        plugin_capability: PluginCapability,
    ) -> bool {
        self.metadata().has_plugin_capability(plugin_capability)
    }
}
