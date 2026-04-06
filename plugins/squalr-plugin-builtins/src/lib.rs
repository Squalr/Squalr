use std::sync::Arc;

use squalr_engine_api::plugins::{data_type::DataTypePlugin, memory_view::MemoryViewPlugin};
use squalr_plugin_data_types_24bit::TwentyFourBitDataTypesPlugin;
use squalr_plugin_memory_view_dolphin::DolphinMemoryViewPlugin;

pub fn get_builtin_memory_view_plugins() -> Vec<Arc<dyn MemoryViewPlugin>> {
    vec![Arc::new(DolphinMemoryViewPlugin::new())]
}

pub fn get_builtin_data_type_plugins() -> Vec<Arc<dyn DataTypePlugin>> {
    vec![Arc::new(TwentyFourBitDataTypesPlugin::new())]
}

#[cfg(test)]
mod tests {
    use super::{get_builtin_data_type_plugins, get_builtin_memory_view_plugins};

    #[test]
    fn builtins_include_dolphin_memory_view_plugin() {
        let plugins = get_builtin_memory_view_plugins();

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].metadata().get_plugin_id(), "builtin.memory-view.dolphin");
    }

    #[test]
    fn builtins_include_24_bit_data_type_plugin() {
        let plugins = get_builtin_data_type_plugins();

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].metadata().get_plugin_id(), "builtin.data-type.24bit-integers");
    }
}
