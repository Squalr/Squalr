use std::sync::Arc;

use squalr_engine_api::{plugins::memory_view::MemoryViewPlugin, structures::processes::opened_process_info::OpenedProcessInfo};
use squalr_plugin_builtins::get_builtin_memory_view_plugins;

pub struct PluginRegistry {
    memory_view_plugins: Vec<Arc<dyn MemoryViewPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            memory_view_plugins: get_builtin_memory_view_plugins(),
        }
    }

    pub fn get_memory_view_plugins(&self) -> &[Arc<dyn MemoryViewPlugin>] {
        &self.memory_view_plugins
    }

    pub fn find_memory_view_plugin(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Option<Arc<dyn MemoryViewPlugin>> {
        self.memory_view_plugins
            .iter()
            .find(|memory_view_plugin| memory_view_plugin.can_attach(process_info))
            .cloned()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::PluginRegistry;
    use squalr_engine_api::structures::{memory::bitness::Bitness, processes::opened_process_info::OpenedProcessInfo};

    #[test]
    fn registry_exposes_builtin_dolphin_memory_view_plugin() {
        let plugin_registry = PluginRegistry::new();
        let opened_process_info = OpenedProcessInfo::new(1, "Dolphin.exe".to_string(), 0, Bitness::Bit64, None);
        let plugin = plugin_registry.find_memory_view_plugin(&opened_process_info);

        assert!(plugin.is_some());
        assert_eq!(plugin_registry.get_memory_view_plugins().len(), 1);
        assert_eq!(
            plugin
                .expect("Expected the Dolphin plugin to match the Dolphin process.")
                .metadata()
                .get_plugin_id(),
            "builtin.memory-view.dolphin"
        );
    }
}
