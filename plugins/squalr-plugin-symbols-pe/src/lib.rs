mod constants;
mod plugin;
mod populate_pe_symbols_action;

pub use plugin::PeSymbolsPlugin;

#[cfg(test)]
mod tests {
    use super::PeSymbolsPlugin;
    use squalr_engine_api::plugins::{Plugin, PluginPermission, symbol_tree::symbol_tree_plugin::SymbolTreePlugin};

    #[test]
    fn plugin_exposes_symbol_store_and_symbol_tree_permissions() {
        let plugin = PeSymbolsPlugin::new();

        assert_eq!(plugin.metadata().get_plugin_id(), "builtin.symbols.pe");
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
