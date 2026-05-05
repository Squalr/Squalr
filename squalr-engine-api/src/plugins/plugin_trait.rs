use crate::plugins::{PluginCapability, PluginMetadata, PluginPermission};

pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;

    fn supports_capability(
        &self,
        plugin_capability: PluginCapability,
    ) -> bool {
        self.metadata().has_plugin_capability(plugin_capability)
    }

    fn has_permission(
        &self,
        plugin_permission: PluginPermission,
    ) -> bool {
        self.metadata().has_plugin_permission(plugin_permission)
    }
}
