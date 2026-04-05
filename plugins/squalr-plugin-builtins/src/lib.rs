use std::sync::Arc;

use squalr_engine_api::plugins::memory_view::MemoryViewPlugin;
use squalr_plugin_memory_view_dolphin::DolphinMemoryViewPlugin;

pub fn get_builtin_memory_view_plugins() -> Vec<Arc<dyn MemoryViewPlugin>> {
    vec![Arc::new(DolphinMemoryViewPlugin::new())]
}

#[cfg(test)]
mod tests {
    use super::get_builtin_memory_view_plugins;

    #[test]
    fn builtins_include_dolphin_memory_view_plugin() {
        let plugins = get_builtin_memory_view_plugins();

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].metadata().get_plugin_id(), "builtin.memory-view.dolphin");
    }
}
