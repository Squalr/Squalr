use std::sync::Arc;

use squalr_engine_api::plugins::PluginPackage;
use squalr_plugin_data_types_24bit::TwentyFourBitDataTypesPlugin;
use squalr_plugin_memory_view_dolphin::DolphinMemoryViewPlugin;

pub fn get_builtin_plugin_packages() -> Vec<Arc<dyn PluginPackage>> {
    vec![
        Arc::new(DolphinMemoryViewPlugin::new()),
        Arc::new(TwentyFourBitDataTypesPlugin::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::get_builtin_plugin_packages;
    use squalr_engine_api::plugins::PluginCapability;

    #[test]
    fn builtins_include_dolphin_memory_view_plugin_package() {
        let plugins = get_builtin_plugin_packages();
        let plugin = plugins
            .iter()
            .find(|plugin| plugin.metadata().get_plugin_id() == "builtin.memory-view.dolphin")
            .expect("Expected the Dolphin memory-view package to be registered.");

        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::MemoryView)
        );
    }

    #[test]
    fn builtins_include_24_bit_data_type_plugin_package() {
        let plugins = get_builtin_plugin_packages();
        let plugin = plugins
            .iter()
            .find(|plugin| plugin.metadata().get_plugin_id() == "builtin.data-type.24bit-integers")
            .expect("Expected the 24-bit data-type package to be registered.");

        assert!(
            plugin
                .metadata()
                .has_plugin_capability(PluginCapability::DataType)
        );
    }
}
