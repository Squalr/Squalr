mod constants;
mod formats;
mod plugin;
mod populate_binary_symbols_action;

pub use plugin::BinarySymbolsPlugin;

#[cfg(test)]
mod tests {
    use super::BinarySymbolsPlugin;
    use squalr_engine_api::plugins::{Plugin, PluginPermission, symbol_tree::symbol_tree_plugin::SymbolTreePlugin};

    #[test]
    fn plugin_exposes_symbol_store_and_symbol_tree_permissions() {
        let plugin = BinarySymbolsPlugin::new();

        assert_eq!(plugin.metadata().get_plugin_id(), "builtin.symbols.binary");
        assert!(plugin.metadata().get_is_enabled_by_default());
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::ReadSymbolStore)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::WriteSymbolStore)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::ReadSymbolTreeWindow)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::WriteSymbolTreeWindow)
        );
        assert!(
            plugin
                .metadata()
                .has_plugin_permission(PluginPermission::ReadProcessMemory)
        );
        assert_eq!(plugin.symbol_tree_actions().len(), 1);
    }
}
