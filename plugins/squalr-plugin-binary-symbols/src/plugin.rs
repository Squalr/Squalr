use crate::{
    constants::{BINARY_SYMBOLS_PLUGIN_DESCRIPTION, BINARY_SYMBOLS_PLUGIN_DISPLAY_NAME, BINARY_SYMBOLS_PLUGIN_ID},
    populate_binary_symbols_action::PopulateBinarySymbolsAction,
};
use squalr_engine_api::plugins::{
    Plugin, PluginCapability, PluginMetadata, PluginPackage, PluginPermission, symbol_tree::symbol_tree_action::SymbolTreeAction,
    symbol_tree::symbol_tree_plugin::SymbolTreePlugin,
};
use std::sync::Arc;

pub struct BinarySymbolsPlugin {
    metadata: PluginMetadata,
    symbol_tree_actions: Vec<Arc<dyn SymbolTreeAction>>,
}

impl BinarySymbolsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new_with_permissions(
                BINARY_SYMBOLS_PLUGIN_ID,
                BINARY_SYMBOLS_PLUGIN_DISPLAY_NAME,
                BINARY_SYMBOLS_PLUGIN_DESCRIPTION,
                vec![PluginCapability::SymbolTree],
                vec![
                    PluginPermission::ReadSymbolStore,
                    PluginPermission::WriteSymbolStore,
                    PluginPermission::ReadSymbolTreeWindow,
                    PluginPermission::WriteSymbolTreeWindow,
                    PluginPermission::ReadProcessMemory,
                ],
                true,
                true,
            ),
            symbol_tree_actions: vec![Arc::new(PopulateBinarySymbolsAction)],
        }
    }
}

impl Default for BinarySymbolsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for BinarySymbolsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

impl PluginPackage for BinarySymbolsPlugin {
    fn as_symbol_tree_plugin(&self) -> Option<&dyn SymbolTreePlugin> {
        Some(self)
    }
}

impl SymbolTreePlugin for BinarySymbolsPlugin {
    fn symbol_tree_actions(&self) -> &[Arc<dyn SymbolTreeAction>] {
        &self.symbol_tree_actions
    }
}
