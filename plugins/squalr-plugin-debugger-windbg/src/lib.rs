mod backend;
mod constants;
mod plugin;
mod session;

pub use plugin::WindbgDebuggerPlugin;

#[cfg(test)]
mod tests {
    use super::WindbgDebuggerPlugin;
    use squalr_engine_api::plugins::{Plugin, PluginCapability, PluginPermission};

    #[test]
    fn plugin_exposes_debugger_capability_and_permissions() {
        let plugin = WindbgDebuggerPlugin::new();

        assert_eq!(plugin.metadata().get_plugin_id(), "builtin.debugger.windbg");
        assert!(plugin.metadata().get_is_enabled_by_default());
        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::Debugger)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::AttachDebugger)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::ControlDebuggerExecution)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::ManageDebuggerBreakpoints)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::ReadRegisters)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::WriteRegisters)
        );
    }
}
