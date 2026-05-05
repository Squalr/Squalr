use crate::{
    constants::{PE_SYMBOLS_PLUGIN_DESCRIPTION, PE_SYMBOLS_PLUGIN_DISPLAY_NAME, PE_SYMBOLS_PLUGIN_ID},
    populate_pe_symbols_action::PopulatePeSymbolsAction,
};
use squalr_engine_api::plugins::{
    Plugin, PluginCapability, PluginMetadata, PluginPackage, PluginPermission, symbol_tree::symbol_tree_action::SymbolTreeAction,
    symbol_tree::symbol_tree_plugin::SymbolTreePlugin,
};
use std::sync::Arc;

pub struct PeSymbolsPlugin {
    metadata: PluginMetadata,
    symbol_tree_actions: Vec<Arc<dyn SymbolTreeAction>>,
}

impl PeSymbolsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new_with_permissions(
                PE_SYMBOLS_PLUGIN_ID,
                PE_SYMBOLS_PLUGIN_DISPLAY_NAME,
                PE_SYMBOLS_PLUGIN_DESCRIPTION,
                vec![PluginCapability::SymbolTree],
                vec![
                    PluginPermission::ReadSymbolStore,
                    PluginPermission::WriteSymbolStore,
                    PluginPermission::ReadSymbolTreeWindow,
                    PluginPermission::WriteSymbolTreeWindow,
                ],
                true,
                true,
            ),
            symbol_tree_actions: vec![Arc::new(PopulatePeSymbolsAction)],
        }
    }
}

impl Default for PeSymbolsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PeSymbolsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

impl PluginPackage for PeSymbolsPlugin {
    fn as_symbol_tree_plugin(&self) -> Option<&dyn SymbolTreePlugin> {
        Some(self)
    }
}

impl SymbolTreePlugin for PeSymbolsPlugin {
    fn symbol_tree_actions(&self) -> &[Arc<dyn SymbolTreeAction>] {
        &self.symbol_tree_actions
    }
}
